use crate::clients::agent_gus_bridge::{HilbertLedgerIPCClient, LedgerResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhysicalTrajectoryState {
    pub frame_id: u64,
    pub energy_amplitude: f64,
    pub phase_angle: f64,
}

pub struct GUSSimulationEngine {
    ipc_client: HilbertLedgerIPCClient,
}

impl GUSSimulationEngine {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            ipc_client: HilbertLedgerIPCClient::new(socket_path, "GUS"),
        }
    }

    pub async fn commit_simulation_frame(
        &self,
        frame: PhysicalTrajectoryState,
    ) -> Result<LedgerResponse, Box<dyn std::error::Error>> {
        // Force odd mode indexing (|2n - 1>) required by Hilbert Ledger consensus kernel
        let raw_index = (frame.frame_id % 50) as usize;
        let mode_target = (2 * raw_index) + 1; 

        let re = frame.energy_amplitude * frame.phase_angle.cos();
        let im = frame.energy_amplitude * frame.phase_angle.sin();

        let response = self.ipc_client.inject_eap(mode_target, re, im).await?;
        Ok(response)
    }
}