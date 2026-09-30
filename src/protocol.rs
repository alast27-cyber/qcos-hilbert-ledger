use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum LedgerCommand {
    CheckConsensus,
    CommitState { agent_id: String, state_hash: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusResponse {
    pub status: String,
    pub consensus_score: Option<f64>,
    pub message: String,
}