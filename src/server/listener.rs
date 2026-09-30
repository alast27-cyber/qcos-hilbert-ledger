use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::net::windows::named_pipe::ServerOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipe_name = r"\\.\pipe\qcos_hilbert_ledger";
    
    // Flag for initial creation
    let mut first = true;

    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(first)
            .create(pipe_name)?;

        first = false;

        // Wait for a client to connect
        server.connect().await?;

        // Spawn a task to handle the connection concurrently
        tokio::spawn(async move {
            handle_client(server).await;
        });
    }
}