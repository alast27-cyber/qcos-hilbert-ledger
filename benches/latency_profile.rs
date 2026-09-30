use std::time::Instant;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipe_name = r"\\.\pipe\qcos_hilbert_ledger";
    let iterations = 10_000;
    let mut client = ClientOptions::new().open(pipe_name)?;
    
    let sample_payload = vec![0u8; 256]; // Simulated QNN Attention Head Tensor Frame
    let mut response_buf = vec![0u8; 256];

    let start = Instant::now();
    for _ in 0..iterations {
        client.write_all(&sample_payload).await?;
        client.read_exact(&mut response_buf).await?;
    }
    let duration = start.elapsed();

    println!("====================================================");
    println!(" QCOS HILBERT LEDGER: END-TO-END LATENCY PROFILE");
    println!("====================================================");
    println!(" Total Iterations : {}", iterations);
    println!(" Total Time       : {:?}", duration);
    println!(" Avg Latency/Op   : {:?}", duration / iterations);
    println!(" Throughput       : {:.2} reqs/sec", iterations as f64 / duration.as_secs_f64());
    println!("====================================================");

    Ok(())
}