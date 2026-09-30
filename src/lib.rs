use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub mod clients;
pub mod ledger;

// Re-export HilbertLedgerIPCClient and alias it as HilbertLedgerClient
pub use clients::agent_gus_bridge::{HilbertConnectionPool, HilbertLedgerIPCClient};
pub type HilbertLedgerClient = HilbertLedgerIPCClient;

// Protocol module alias for test/binary compatibility
pub mod protocol {
    pub use super::{ConsensusResponse, LedgerCommand};
}

#[derive(Clone, Debug)]
pub struct TokenBucket {
    #[allow(dead_code)]
    capacity: usize,
    #[allow(dead_code)]
    refill_rate: usize,
    tokens: Arc<Mutex<usize>>,
}

impl TokenBucket {
    pub fn new(capacity: usize, refill_rate: usize) -> Self {
        Self {
            capacity,
            refill_rate,
            tokens: Arc::new(Mutex::new(capacity)),
        }
    }

    pub fn try_acquire(&self, count: usize) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        if *tokens >= count {
            *tokens -= count;
            true
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerCommand {
    CheckConsensus,
    CommitState {
        agent_id: String,
        state_hash: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusResponse {
    pub status: String,
    pub consensus_score: Option<f64>,
    pub message: String,
}
