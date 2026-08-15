pub mod batch;
pub mod session;

pub use batch::*;
pub use session::*;

use jio_consensus::consensus::ctl::ConsensusCtl;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConsensusManager {
    consensus: Arc<RwLock<Arc<dyn ConsensusCtl>>>,
}

impl ConsensusManager {
    pub fn new(consensus: Arc<dyn ConsensusCtl>) -> Self {
        Self {
            consensus: Arc::new(RwLock::new(consensus)),
        }
    }

    pub fn session(&self) -> ConsensusSession {
        ConsensusSession::new(self.consensus.read().clone())
    }

    pub fn consensus(&self) -> Arc<dyn ConsensusCtl> {
        self.consensus.read().clone()
    }

    pub fn set_consensus(&self, consensus: Arc<dyn ConsensusCtl>) {
        *self.consensus.write() = consensus;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus::consensus::factory::ConsensusFactory;
    use jio_consensus_core::config::params::Params;

    #[test]
    fn test_consensus_manager_session() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let mgr = ConsensusManager::new(consensus);

        let session = mgr.session();
        let vs = session.get_virtual_state().expect("virtual state exists");
        assert_eq!(vs.parents.len(), 1);
    }
}
