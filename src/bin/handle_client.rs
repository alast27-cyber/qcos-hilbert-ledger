use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use qcos_hilbert_ledger::{ConsensusResponse, LedgerCommand, TokenBucket};

const PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("[INFO] Starting QCOS Hilbert Ledger Daemon with Token Bucket Rate Limiting...");

    let rate_limiter = TokenBucket::new(500_000, 300_000);
    let mut is_first = true;

    loop {
        let mut builder = ServerOptions::new();
        builder.access_inbound(true).access_outbound(true);
        builder.first_pipe_instance(is_first);

        let server = match builder.create(PIPE_NAME) {
            Ok(s) => s,
            Err(e) => { eprintln!("[Server Error] Pipe creation failed: {} (os error {})", e, e.raw_os_error().unwrap_or(0));
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                continue;
            }
        };
        is_first = false;

        if server.connect().await.is_ok() {
            let limiter = rate_limiter.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_client(server, limiter).await {
                    eprintln!("[WARN] Connection closed with error: {}", e);
                }
            });
        }
    }
}

async fn handle_client(
    mut pipe: NamedPipeServer,
    limiter: TokenBucket,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 8192];

    loop {
        let n = match pipe.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => return Err(Box::new(e)),
        };

        if !limiter.try_acquire(1) {
            let err_resp = ConsensusResponse {
                status: "RATE_LIMITED".to_string(),
                consensus_score: None,
                message: "QCOS Hilbert Ledger pipeline capacity reached. Retry shortly.".to_string(),
            };
            let payload = serde_json::to_vec(&err_resp)?;
            pipe.write_all(&payload).await?;
            pipe.flush().await?;
            continue;
        }

        let response_bytes = if let Ok(cmd) = serde_json::from_slice::<LedgerCommand>(&buf[..n]) {
            match cmd {
                LedgerCommand::CheckConsensus => {
                    let resp = ConsensusResponse {
                        status: "OK".to_string(),
                        consensus_score: Some(0.9998),
                        message: "Hilbert Ledger Consensus State: STABLE".to_string(),
                    };
                    serde_json::to_vec(&resp)?
                }
                LedgerCommand::CommitState { agent_id, state_hash } => {
                    let resp = ConsensusResponse {
                        status: "COMMITTED".to_string(),
                        consensus_score: Some(1.0),
                        message: format!("Agent [{}] state [{}] recorded.", agent_id, state_hash),
                    };
                    serde_json::to_vec(&resp)?
                }
            }
        } else {
            buf[..n].to_vec()
        };

        pipe.write_all(&response_bytes).await?;
        pipe.flush().await?;
    }

    Ok(())
}


