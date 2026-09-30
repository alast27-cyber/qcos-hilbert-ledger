use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::ClientOptions;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipe_path = r"\\.\pipe\qcos_hilbert_ledger";
    println!("Connecting to {}...", pipe_path);

    match ClientOptions::new().open(pipe_path) {
        Ok(mut stream) => {
            println!("Successfully opened pipe handle!");
            let req = json!({
                "action": "verify_consensus"
            });
            let payload = serde_json::to_vec(&req)?;

            println!("Sending payload: {}", String::from_utf8_lossy(&payload));
            stream.write_all(&payload).await?;

            let mut buf = vec![0u8; 1024];
            match stream.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    println!("Response ({}) bytes): {}", n, String::from_utf8_lossy(&buf[..n]));
                }
                Ok(_) => println!("Received 0 bytes (server closed connection immediately)."),
                Err(e) => println!("Read error: {:?}", e),
            }
        }
        Err(e) => {
            println!("Failed to open named pipe: {:?}", e);
        }
    }

    Ok(())
}
