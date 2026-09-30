use qcos_hilbert_ledger::clients::agent_gus_bridge::HilbertLedgerIPCClient;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = r"\\.\pipe\qcos_hilbert_ledger";
    let monitor_client = HilbertLedgerIPCClient::new(socket_path, "QCOS_Monitor");

    println!("====================================================");
    println!("     QCOS DQHHL DAEMON REAL-TIME MONITOR           ");
    println!("====================================================");

    loop {
        match monitor_client.check_consensus().await {
            Ok(response) => {
                let score = response.consensus_score.unwrap_or(0.0);
                print!("\r\x1B[KStatus: {} | Consensus Score: {:.6} | Msg: {}", 
                    response.status, score, response.message);
                io::stdout().flush()?;
            }
            Err(e) => {
                print!("\r\x1B[K[OFFLINE] Could not connect to Hilbert Ledger daemon: {}", e);
                io::stdout().flush()?;
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
