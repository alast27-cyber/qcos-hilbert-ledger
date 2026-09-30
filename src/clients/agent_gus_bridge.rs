use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

#[derive(Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum LedgerRequest {
    Inject {
        client_id: String,
        mode_index: usize,
        amplitude_re: f64,
        amplitude_im: f64,
    },
    Shift,
    VerifyConsensus,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LedgerResponse {
    pub status: String,
    pub message: String,
    pub consensus_score: Option<f64>,
}

struct PersistentIPCChannel {
    pipe_path: String,
    _client_name: String,
    #[cfg(windows)]
    stream: Option<NamedPipeClient>,
}

impl PersistentIPCChannel {
    pub fn new(pipe_path: &str, client_name: &str) -> Self {
        Self {
            pipe_path: pipe_path.to_string(),
            _client_name: client_name.to_string(),
            #[cfg(windows)]
            stream: None,
        }
    }

    async fn ensure_connected(&mut self) -> Result<&mut NamedPipeClient, String> {
        #[cfg(windows)]
        {
            if self.stream.is_none() {
                let client = ClientOptions::new()
                    .open(&self.pipe_path)
                    .map_err(|e| format!("Failed to open named pipe {}: {}", self.pipe_path, e))?;
                self.stream = Some(client);
            }
            Ok(self.stream.as_mut().unwrap())
        }
        #[cfg(unix)]
        {
            Err("UNIX socket handling not implemented in this build".into())
        }
    }

    pub async fn send_request(&mut self, request: &LedgerRequest) -> Result<LedgerResponse, String> {
        let mut payload = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        payload.push(b'\n');

        for attempt in 0..2 {
            let stream = match self.ensure_connected().await {
                Ok(s) => s,
                Err(e) => {
                    if attempt == 1 {
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    continue;
                }
            };

            if stream.write_all(&payload).await.is_ok() {
                let mut line = String::new();
                let mut reader = BufReader::new(&mut *stream);

                match timeout(Duration::from_millis(1000), reader.read_line(&mut line)).await {
                    Ok(Ok(n)) if n > 0 => {
                        if let Ok(resp) = serde_json::from_str::<LedgerResponse>(line.trim()) {
                            return Ok(resp);
                        }
                    }
                    _ => {
                        self.stream = None;
                    }
                }
            } else {
                self.stream = None;
            }

            if attempt == 1 {
                return Err("Failed to complete IPC request round-trip".into());
            }
        }

        Err("Max attempts reached for IPC request".into())
    }
}

pub struct HilbertConnectionPool {
    sender: mpsc::Sender<PersistentIPCChannel>,
    receiver: Arc<Mutex<mpsc::Receiver<PersistentIPCChannel>>>,
}

impl Clone for HilbertConnectionPool {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl HilbertConnectionPool {
    pub async fn new(pipe_path: &str, client_name: &str, capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        for i in 0..capacity {
            let channel = PersistentIPCChannel::new(pipe_path, &format!("{}_{}", client_name, i));
            let _ = sender.send(channel).await;
        }
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    async fn execute(&self, request: LedgerRequest) -> Result<LedgerResponse, String> {
        let mut channel = {
            let mut rx = self.receiver.lock().await;
            rx.recv().await.ok_or_else(|| "Connection pool exhausted or closed".to_string())?
        };

        let result = channel.send_request(&request).await;
        let _ = self.sender.send(channel).await;
        result
    }

    pub async fn inject_eap(&self, client_id: &str, mode_index: usize, re: f64, im: f64) -> Result<LedgerResponse, String> {
        self.execute(LedgerRequest::Inject {
            client_id: client_id.to_string(),
            mode_index,
            amplitude_re: re,
            amplitude_im: im,
        }).await
    }

    pub async fn trigger_shift(&self) -> Result<LedgerResponse, String> {
        self.execute(LedgerRequest::Shift).await
    }

    pub async fn check_consensus(&self) -> Result<LedgerResponse, String> {
        self.execute(LedgerRequest::VerifyConsensus).await
    }
}

pub struct HilbertLedgerIPCClient {
    pipe_path: String,
    client_name: String,
}

impl HilbertLedgerIPCClient {
    pub fn new(pipe_path: impl Into<String>, client_name: impl Into<String>) -> Self {
        Self {
            pipe_path: pipe_path.into(),
            client_name: client_name.into(),
        }
    }

    pub async fn inject_eap(&self, mode_index: usize, re: f64, im: f64) -> Result<LedgerResponse, String> {
        let mut channel = PersistentIPCChannel::new(&self.pipe_path, &self.client_name);
        channel.send_request(&LedgerRequest::Inject {
            client_id: self.client_name.clone(),
            mode_index,
            amplitude_re: re,
            amplitude_im: im,
        }).await
    }

    pub async fn trigger_shift(&self) -> Result<LedgerResponse, String> {
        let mut channel = PersistentIPCChannel::new(&self.pipe_path, &self.client_name);
        channel.send_request(&LedgerRequest::Shift).await
    }

    pub async fn check_consensus(&self) -> Result<LedgerResponse, String> {
        let mut channel = PersistentIPCChannel::new(&self.pipe_path, &self.client_name);
        channel.send_request(&LedgerRequest::VerifyConsensus).await
    }
}
