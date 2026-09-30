#[cfg(test)]
mod system_tests {
    use qcos_hilbert_ledger::ledger::HilbertHotelLedger;

    #[test]
    fn test_agentq_gus_eap_lifecycle() {
        // Restricted access to AgentQ and GUS only
        let clients = vec!["AgentQ".into(), "GUS".into()];
        let mut ledger = HilbertHotelLedger::new(0.01, clients);

        // 1. AgentQ injects reasoning state into mode |1>
        assert!(ledger.eap_teleport_inject("AgentQ", 1, 0.7071, 0.7071).is_ok());

        // 2. GUS commits physical trajectory frame into mode |3>
        assert!(ledger.eap_teleport_inject("GUS", 3, 0.5000, 0.8660).is_ok());

        // Verify occupied states
        assert!(ledger.modes.contains_key(&1));
        assert!(ledger.modes.contains_key(&3));

        // 3. Execute Hilbert Shift Operator |n> -> |2n>
        ledger.execute_hilbert_shift();

        // Check shifted modes (|1> -> |2>, |3> -> |6>)
        assert!(!ledger.modes.contains_key(&1));
        assert!(!ledger.modes.contains_key(&3));
        assert!(ledger.modes.contains_key(&2));
        assert!(ledger.modes.contains_key(&6));

        // 4. AgentQ injects new reasoning state into freshly cleared mode |1>
        assert!(ledger.eap_teleport_inject("AgentQ", 1, 1.0000, 0.0000).is_ok());

        // 5. Calculate phase consensus score
        let consensus = ledger.verify_phase_consensus();
        assert!(consensus > 0.0);
    }
}