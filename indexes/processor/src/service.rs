use crate::processor::IndexProcessor;
use jio_consensusmanager::ConsensusManager;
use std::sync::Arc;

pub struct IndexService {
    #[allow(dead_code)]
    consensus_manager: ConsensusManager,
    processor: Arc<IndexProcessor>,
}

impl IndexService {
    pub fn new(consensus_manager: ConsensusManager, processor: Arc<IndexProcessor>) -> Self {
        Self {
            consensus_manager,
            processor,
        }
    }

    pub fn processor(&self) -> Arc<IndexProcessor> {
        self.processor.clone()
    }
}
