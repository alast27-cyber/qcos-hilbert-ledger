mod ledger;

use ledger::HilbertHotelLedger;
use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    security: SecurityConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    ipc_socket_path: String,
    gamma_decay: f64,
}

#[derive(Deserialize)]
struct SecurityConfig {
    authorized_clients: Vec<String>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum LedgerRequest {
    Inject {
        client_id: String,
        mode_index: usize,
        amplitude_re: f64,
        amplitude_im: f64,
    },
    Shift,
    VerifyConsensus,
}

#[derive(Serialize)]
struct LedgerResponse {
    status: String,
    message: String,
    consensus_score: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load config
    let config_raw = std::fs::read_to_string("config/ledger_daemon.toml")
        .unwrap_or_else(|_| include_str!("../config/ledger_daemon.toml").to_string());
    let config: Config = toml::from_str(&config_raw)?;

    let socket_path = config.server.ipc_socket_path;
    let _ = std::fs::remove_file(&socket_path);

    let ledger = Arc::new(Mutex::new(HilbertHotelLedger::new(
        config.server.gamma_decay,
        config.security.authorized_clients,
    )));

    let listener = UnixListener::bind(&socket_path)?;
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o700))?;

    tracing::info!("Standalone Hilbert Ledger Daemon running on UNIX socket: {}", socket_path);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let ledger_ref = Arc::clone(&ledger);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req: Result<LedgerRequest, _> = serde_json::from_slice(&buf[..n]);
            let response = match req {
                Ok(LedgerRequest::Inject { client_id, mode_index, amplitude_re, amplitude_im }) => {
                    let mut engine = ledger_ref.lock().await;
                    match engine.eap_teleport_inject(&client_id, mode_index, amplitude_re, amplitude_im) {
                        Ok(_) => LedgerResponse {
                            status: "OK".into(),
                            message: format!("EAP injected into mode |{}>", mode_index),
                            consensus_score: None,
                        },
                        Err(e) => LedgerResponse {
                            status: "ERROR".into(),
                            message: e.to_string(),
                            consensus_score: None,
                        },
                    }
                }
                Ok(LedgerRequest::Shift) => {
                    let mut engine = ledger_ref.lock().await;
                    engine.execute_hilbert_shift();
                    LedgerResponse {
                        status: "OK".into(),
                        message: "Shift operation |n> -> |2n> executed successfully.".into(),
                        consensus_score: None,
                    }
                }
                Ok(LedgerRequest::VerifyConsensus) => {
                    let engine = ledger_ref.lock().await;
                    let score = engine.verify_phase_consensus();
                    LedgerResponse {
                        status: "OK".into(),
                        message: "Phase consensus score calculated.".into(),
                        consensus_score: Some(score),
                    }
                }
                Err(_) => LedgerResponse {
                    status: "ERROR".into(),
                    message: "Invalid JSON payload structure.".into(),
                    consensus_score: None,
                },
            };

            let resp_bytes = serde_json::to_vec(&response).unwrap();
            let _ = socket.write_all(&resp_bytes).await;
        });
    }
}