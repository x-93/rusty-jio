use crate::pipeline::body_processor::body_validation_in_context::validate_body_in_context;
use crate::pipeline::body_processor::body_validation_in_isolation::validate_body_in_isolation;
use crate::pipeline::header_processor::HeaderProcessor;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::errors::consensus::ConsensusResult;

pub struct BodyProcessor {
    header_processor: HeaderProcessor,
}

impl BodyProcessor {
    pub fn new(header_processor: HeaderProcessor) -> Self {
        Self { header_processor }
    }

    pub fn process_body(&self, block: &Block) -> ConsensusResult<BlockHash> {
        // 1. Process header first
        let hash = self.header_processor.process_header(&block.header)?;

        // 2. Validate in isolation
        validate_body_in_isolation(block)?;

        // 3. Validate in context
        validate_body_in_context(block)?;

        Ok(hash)
    }
}
