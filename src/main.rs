use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::Semaphore;

const PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";
const MAX_CONCURRENT_CLIENTS: usize = 256;

async fn handle_client(mut stream: NamedPipeServer) {
    let mut buffer = [0u8; 8192];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if stream.write_all(&buffer[..n]).await.is_err() {
                    break;
                }
                if stream.flush().await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("[QCOS Hilbert Ledger] Server initializing on {}", PIPE_NAME);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CLIENTS));
    let mut is_first = true;

    loop {
        let permit = semaphore.clone().acquire_owned().await?;

        let server = match ServerOptions::new()
            .first_pipe_instance(is_first)
            .reject_remote_clients(true)
            .create(PIPE_NAME)
        {
            Ok(s) => {
                is_first = false;
                s
            }
            Err(e) => {
                eprintln!("[Server Error] Pipe creation failed: {}", e);
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                continue;
            }
        };

        // Spawn listener immediately so next pipe instance is created without delay
        tokio::spawn(async move {
            if server.connect().await.is_ok() {
                handle_client(server).await;
            }
            drop(permit);
        });
    }
}