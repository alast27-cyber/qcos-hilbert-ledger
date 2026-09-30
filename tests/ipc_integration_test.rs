#[cfg(test)]
mod tests {
    use qcos_hilbert_ledger::ledger::HilbertHotelLedger;

    #[test]
    fn test_authorized_client_eap_injection() {
        let authorized_clients = vec!["AgentQ".to_string(), "GUS".to_string(), "CLL".to_string()];
        let mut ledger = HilbertHotelLedger::new(0.05, authorized_clients);

        // Test valid injection from AgentQ into odd mode |1>
        let res = ledger.eap_teleport_inject("AgentQ", 1, 0.707, 0.707);
        assert!(res.is_ok());

        // Test rejection of unauthorized client
        let res_unauth = ledger.eap_teleport_inject("ExternalHacker", 3, 1.0, 0.0);
        assert!(res_unauth.is_err());
        assert_eq!(
            res_unauth.unwrap_err(),
            "ACCESS_DENIED: Client not in authorized list (AgentQ, GUS, CLL only)."
        );
    }

    #[test]
    fn test_hilbert_shift_vacates_odd_modes() {
        let authorized_clients = vec!["AgentQ".to_string()];
        let mut ledger = HilbertHotelLedger::new(0.05, authorized_clients);

        // Populate mode |1>
        let _ = ledger.eap_teleport_inject("AgentQ", 1, 0.707, 0.707);
        assert!(ledger.modes.contains_key(&1));

        // Execute shift operator |n> -> |2n>
        ledger.execute_hilbert_shift();

        // Mode |1> is now shifted to |2>
        assert!(!ledger.modes.contains_key(&1));
        assert!(ledger.modes.contains_key(&2));

        // Mode |1> accepts new EAP injection
        let inject_res = ledger.eap_teleport_inject("AgentQ", 1, 0.0, 1.0);
        assert!(inject_res.is_ok());
    }
}
