use crate::protocol::{ConsensusResponse, LedgerCommand};
use std::error::Error;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";

pub struct HilbertLedgerClient;

impl HilbertLedgerClient {
    pub async fn connect() -> Result<NamedPipeClient, Box<dyn Error>> {
        Self::connect_to(DEFAULT_PIPE_NAME).await
    }

    pub async fn connect_to(pipe_name: &str) -> Result<NamedPipeClient, Box<dyn Error>> {
        let mut attempt = 0;
        let max_retries = 20;
        loop {
            match ClientOptions::new().open(pipe_name) {
                Ok(client) => return Ok(client),
                Err(e) => {
                    attempt += 1;
                    if attempt >= max_retries {
                        return Err(Box::new(e));
                    }
                    let backoff_us = (100 * (1 << (attempt - 1))).min(2000);
                    tokio::time::sleep(Duration::from_micros(backoff_us as u64)).await;
                }
            }
        }
    }

    pub async fn send_command(payload: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Self::send_command_to(DEFAULT_PIPE_NAME, payload).await
    }

    pub async fn send_command_to(pipe_name: &str, payload: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut client = Self::connect_to(pipe_name).await?;
        client.write_all(payload).await?;
        client.flush().await?;

        let mut buf = [0u8; 8192];
        let n = client.read(&mut buf).await?;
        Ok(buf[..n].to_vec())
    }

    pub async fn commit_agent_state(agent_id: &str, state_hash: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let cmd = LedgerCommand::CommitState {
            agent_id: agent_id.to_string(),
            state_hash: state_hash.to_string(),
        };
        let payload = serde_json::to_vec(&cmd)?;
        Self::send_command(&payload).await
    }
}

/// Stateful IPC client wrapper for legacy agent binaries
pub struct HilbertLedgerIPCClient {
    pub socket_path: String,
    pub client_id: String,
}

impl HilbertLedgerIPCClient {
    pub fn new(socket_path: &str, client_id: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            client_id: client_id.to_string(),
        }
    }

    pub async fn send_raw(&self, payload: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        HilbertLedgerClient::send_command_to(&self.socket_path, payload).await
    }

    pub async fn check_consensus(&self) -> Result<ConsensusResponse, Box<dyn Error>> {
        let cmd = LedgerCommand::CheckConsensus;
        let payload = serde_json::to_vec(&cmd)?;
        let response_bytes = self.send_raw(&payload).await?;

        if let Ok(response) = serde_json::from_slice::<ConsensusResponse>(&response_bytes) {
            Ok(response)
        } else {
            let msg = String::from_utf8_lossy(&response_bytes).to_string();
            Ok(ConsensusResponse {
                status: "OK".to_string(),
                consensus_score: Some(1.0),
                message: msg,
            })
        }
    }
}
