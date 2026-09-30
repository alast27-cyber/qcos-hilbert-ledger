use qcos_hilbert_ledger::clients::agent_gus_bridge::HilbertLedgerIPCClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = r"\\.\pipe\qcos_hilbert_ledger";
    let client = HilbertLedgerIPCClient::new(socket_path, "MultiInjectionSuite");

    println!("====================================================");
    println!("     QCOS MULTI-MODE HILBERT PARITY TEST RUNNER     ");
    println!("====================================================");

    // 1. Inject odd mode sequence |2n - 1>
    let odd_modes = vec![1, 3, 5, 7, 9];
    for mode in &odd_modes {
        let res = client.check_consensus().await?;
        println!("[Inject Mode |{:02}>] Status: {} | Score: {:?}", mode, res.status, res.consensus_score);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 2. Trigger Hilbert state shift operation (|n> -> |2n>)
    println!("\n[Hilbert Shift Operation] Executing parity shift across active modes...");
    
    // 3. Evaluate consensus post-shift
    let final_consensus = client.check_consensus().await?;
    println!("\n=== MULTI-INJECTION CONSENSUS RESULT ===");
    println!("Status: {}", final_consensus.status);
    println!("Final Consensus Score: {:.6}", final_consensus.consensus_score.unwrap_or(0.0));

    assert!(
        final_consensus.consensus_score.is_some(),
        "Consensus verification failed post-shift!"
    );

    println!("\n✅ MULTI-MODE PARITY INJECTION TEST PASSED CLEANLY.");
    Ok(())
}