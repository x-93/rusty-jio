use jio_consensus::consensus::ctl::ConsensusCtl;
use std::ops::Deref;
use std::sync::Arc;

pub struct ConsensusSession {
    consensus: Arc<dyn ConsensusCtl>,
}

impl ConsensusSession {
    pub fn new(consensus: Arc<dyn ConsensusCtl>) -> Self {
        Self { consensus }
    }
}

impl Deref for ConsensusSession {
    type Target = Arc<dyn ConsensusCtl>;

    fn deref(&self) -> &Self::Target {
        &self.consensus
    }
}
