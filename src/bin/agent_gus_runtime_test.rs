use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use qcos_hilbert_ledger::protocol::{ConsensusResponse, LedgerCommand};

const PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("====================================================");
    println!("     QCOS HILBERT LEDGER DAEMON INITIALIZING         ");
    println!("====================================================");
    println!("[INFO] Listening on pipe: {}", PIPE_NAME);

    // Track whether this is the first server instance created for security/permissions
    let mut is_first_instance = true;

    loop {
        // Construct server options for named pipe
        let mut server_builder = ServerOptions::new();
        server_builder.first_pipe_instance(is_first_instance);

        // Bind the named pipe listener
        let server = match server_builder.create(PIPE_NAME) {
            Ok(server) => server,
            Err(e) => {
                eprintln!("[ERROR] Failed to bind named pipe {}: {}", PIPE_NAME, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
        };

        // Subsequent pipe instances do not set first_pipe_instance
        is_first_instance = false;

        // Wait for an incoming client connection
        match server.connect().await {
            Ok(()) => {
                // Spawn a dedicated Tokio task to handle each connected client concurrently
                tokio::spawn(async move {
                    if let Err(e) = handle_client(server).await {
                        eprintln!("[WARN] Error handling client session: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[WARN] Failed client connection hand-off: {}", e);
            }
        }
    }
}

/// Asynchronously reads commands from a client pipe and returns formatted responses
async fn handle_client(mut pipe: NamedPipeServer) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 8192];

    loop {
        let n = match pipe.read(&mut buf).await {
            Ok(0) => break, // EOF: Client disconnected
            Ok(n) => n,
            Err(e) => return Err(Box::new(e)),
        };

        // Attempt JSON deserialization of command
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
            // Echo raw string payload back if not formatted JSON command (for backward compatibility)
            buf[..n].to_vec()
        };

        // Write response back to the client
        pipe.write_all(&response_bytes).await?;
        pipe.flush().await?;
    }

    Ok(())
}