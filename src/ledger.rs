use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HilbertStateMode {
    pub mode_index: usize,
    pub amplitude_re: f64,
    pub amplitude_im: f64,
}

#[derive(Debug)]
pub struct HilbertHotelLedger {
    pub modes: HashMap<usize, HilbertStateMode>,
    pub gamma_decay: f64,
    pub authorized_clients: Vec<String>,
}

impl HilbertHotelLedger {
    pub fn new(gamma: f64, authorized_clients: Vec<String>) -> Self {
        Self {
            modes: HashMap::new(),
            gamma_decay: gamma,
            authorized_clients,
        }
    }

    /// Shift operator S: maps active modes |n> -> |2n>, vacating all odd modes |2n-1>
    pub fn execute_hilbert_shift(&mut self) {
        let damping_factor = (-self.gamma_decay).exp();
        let mut shifted_modes = HashMap::new();

        for (n, mode) in self.modes.drain() {
            let new_index = 2 * n;
            shifted_modes.insert(new_index, HilbertStateMode {
                mode_index: new_index,
                amplitude_re: mode.amplitude_re * damping_factor,
                amplitude_im: mode.amplitude_im * damping_factor,
            });
        }
        self.modes = shifted_modes;
    }

    /// Entanglement Access Protocol (EAP) Injection into vacant odd mode |2n-1>
    pub fn eap_teleport_inject(
        &mut self,
        client_id: &str,
        target_odd_mode: usize,
        re: f64,
        im: f64,
    ) -> Result<(), &'static str> {
        if !self.authorized_clients.iter().any(|c| c == client_id) {
            return Err("ACCESS_DENIED: Client not in authorized list (AgentQ, GUS, CLL only).");
        }
        if target_odd_mode % 2 == 0 {
            return Err("EAP Error: Target mode must be odd-numbered (|2n-1>).");
        }
        if self.modes.contains_key(&target_odd_mode) {
            return Err("EAP Error: Target odd mode is occupied.");
        }

        self.modes.insert(target_odd_mode, HilbertStateMode {
            mode_index: target_odd_mode,
            amplitude_re: re,
            amplitude_im: im,
        });

        Ok(())
    }

    pub fn verify_phase_consensus(&self) -> f64 {
        if self.modes.is_empty() { return 1.0; }
        let total_amplitude: f64 = self.modes.values()
            .map(|m| (m.amplitude_re.powi(2) + m.amplitude_im.powi(2)).sqrt())
            .sum();
        total_amplitude / (self.modes.len() as f64)
    }
}