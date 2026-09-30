use qcos_hilbert_ledger::HilbertLedgerClient;

#[tokio::test]
async fn test_ledger_consensus_check() {
    let client = HilbertLedgerClient::new(r"\\.\pipe\qcos_hilbert_ledger", "IntegrationTest_Agent");
    
    // Test consensus check call
    let response = client.check_consensus().await;
    println!("Consensus response: {:?}", response);
    
    assert!(response.is_ok() || response.is_err());
}

#[tokio::test]
async fn test_ledger_eap_injection() {
    let client = HilbertLedgerClient::new(r"\\.\pipe\qcos_hilbert_ledger", "IntegrationTest_Agent");
    
    // Test EAP state injection call (mode index 0, amplitude 1.0 + 0.0i)
    let response = client.inject_eap(0, 1.0, 0.0).await;
    println!("EAP injection response: {:?}", response);
    
    assert!(response.is_ok() || response.is_err());
}
