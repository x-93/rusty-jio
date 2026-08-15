use jio_consensus::consensus::factory::ConsensusFactory;
use jio_consensus_core::config::params::Params;
use jio_consensusmanager::ConsensusManager;

pub struct TestConsensusFixture {
    pub consensus_manager: ConsensusManager,
}

impl TestConsensusFixture {
    pub fn new() -> Self {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_manager = ConsensusManager::new(consensus);
        Self { consensus_manager }
    }
}
