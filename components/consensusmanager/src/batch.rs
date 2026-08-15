use jio_consensus::consensus::ctl::ConsensusCtl;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::errors::consensus::ConsensusResult;
use std::sync::Arc;

pub struct ConsensusBatchExecutor;

impl ConsensusBatchExecutor {
    pub fn execute_batch(
        consensus: &Arc<dyn ConsensusCtl>,
        blocks: Vec<Block>,
    ) -> Vec<ConsensusResult<BlockHash>> {
        blocks
            .into_iter()
            .map(|block| consensus.validate_and_insert_block(block))
            .collect()
    }
}
