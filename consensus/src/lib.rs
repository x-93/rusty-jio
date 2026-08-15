pub mod config;
pub mod consensus;
pub mod errors;
pub mod model;
pub mod pipeline;
pub mod processes;
pub mod test_helpers;

pub use config::*;
pub use consensus::*;
pub use errors::*;

#[cfg(test)]
mod tests {
    use crate::consensus::factory::ConsensusFactory;
    use crate::test_helpers::create_test_block;
    use jio_consensus_core::config::params::Params;

    #[test]
    fn test_consensus_initialization_and_block_insertion() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);

        // Genesis should already be in state
        let vs = consensus.get_virtual_state().expect("virtual state exists");
        assert_eq!(vs.parents.len(), 1);

        let genesis_hash = vs.parents[0];
        let status = consensus.get_status(&genesis_hash);
        assert!(status.is_some());

        // Submit child block
        let block1 = create_test_block(
            vec![genesis_hash],
            jio_core::time::unix_now(),
            0x207f_ffff,
            1,
        );

        let block1_hash = consensus
            .validate_and_insert_block(block1)
            .expect("block 1 insertion succeeds");

        assert_eq!(consensus.get_selected_chain_tip(), Some(block1_hash));
    }

    #[test]
    fn test_dag_branching_and_chain_reorg() {
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];

        // Branch A: Block A1
        let block_a1 = create_test_block(
            vec![genesis_hash],
            jio_core::time::unix_now(),
            0x207f_ffff,
            1,
        );
        let a1_hash = consensus.validate_and_insert_block(block_a1).unwrap();
        assert_eq!(consensus.get_selected_chain_tip(), Some(a1_hash));

        // Branch B: Block B1
        let block_b1 = create_test_block(
            vec![genesis_hash],
            jio_core::time::unix_now() + 10,
            0x207f_ffff,
            1,
        );
        let b1_hash = consensus.validate_and_insert_block(block_b1).unwrap();

        // Branch B: Block B2 (extends B1 -> higher blue work)
        let block_b2 = create_test_block(
            vec![b1_hash],
            jio_core::time::unix_now() + 20,
            0x207f_ffff,
            2,
        );
        let b2_hash = consensus.validate_and_insert_block(block_b2).unwrap();

        // Selected tip must reorganize to B2
        assert_eq!(consensus.get_selected_chain_tip(), Some(b2_hash));
    }
}
