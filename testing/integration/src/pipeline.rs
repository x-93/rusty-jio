use jio_consensus::consensus::factory::ConsensusFactory;
use jio_consensus_core::block::Block;
use jio_consensus_core::config::params::Params;
use jio_consensus_core::header::Header;
use jio_consensusmanager::ConsensusManager;

pub struct PipelineFixture {
    pub consensus_manager: ConsensusManager,
}

impl PipelineFixture {
    pub fn new() -> Self {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_manager = ConsensusManager::new(consensus);
        Self { consensus_manager }
    }

    pub fn validate_and_insert_block(&self, block: Block) -> Result<jio_consensus_core::blockhash::BlockHash, String> {
        let session = self.consensus_manager.session();
        session.validate_and_insert_block(block).map_err(|e| e.to_string())
    }

    pub fn validate_and_insert_header(&self, header: &Header) -> Result<jio_consensus_core::blockhash::BlockHash, String> {
        let session = self.consensus_manager.session();
        session.validate_and_insert_header(header).map_err(|e| e.to_string())
    }
}
