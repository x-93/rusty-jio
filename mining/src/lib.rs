pub mod block_template;
pub mod manager;
pub mod mempool;
pub mod model;

pub use block_template::*;
pub use manager::*;
pub use mempool::*;
pub use model::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus::consensus::factory::ConsensusFactory;
    use jio_consensus_core::config::params::Params;
    use jio_consensus_core::tx::ScriptPublicKey;
    use jio_consensusmanager::ConsensusManager;

    #[test]
    fn test_mining_manager_block_template() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_mgr = ConsensusManager::new(consensus);
        let mining_mgr = MiningManager::new(consensus_mgr, MempoolConfig::default());

        let payee = ScriptPublicKey::from_vec(0, vec![1, 2, 3, 4]);
        let template = mining_mgr
            .get_block_template(payee, vec![])
            .expect("template build success");

        assert_eq!(template.block.transactions.len(), 1); // Coinbase
        assert!(template.block.header.timestamp > 0);
    }
}
