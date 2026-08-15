use crate::consensus::factory::ConsensusFactory;
use crate::pipeline::deps_manager::BlockTaskDependencyManager;
use crate::pipeline::virtual_processor::test_block_builder::TestBlockBuilder;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::config::params::Params;
use jio_consensus_core::hashing::header::header_hash;
use jio_math::Uint192;

#[test]
fn test_block_status_transitions_header_to_body() {
    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let now = jio_core::time::unix_now();

    // 1. Create a valid test block
    let block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .build();
    let header = block.header.clone();
    let expected_hash = header_hash(&header);

    // 2. Insert only the header first
    let header_hash_res = consensus.validate_and_insert_header(&header).expect("valid header insertion");
    assert_eq!(header_hash_res, expected_hash);

    // Block status should be HeaderOnly
    let status_after_header = consensus.get_status(&expected_hash);
    assert_eq!(status_after_header, Some(BlockStatus::StatusHeaderOnly));
    assert!(status_after_header.unwrap().is_header_only());
    assert!(!status_after_header.unwrap().is_utxo_valid());

    // 3. Now insert the full block (header + body)
    let block_hash_res = consensus.validate_and_insert_block(block).expect("valid block body insertion");
    assert_eq!(block_hash_res, expected_hash);

    // Block status should transition to StatusUTXOValid
    let status_after_block = consensus.get_status(&expected_hash);
    assert_eq!(status_after_block, Some(BlockStatus::StatusUTXOValid));
    assert!(!status_after_block.unwrap().is_header_only());
    assert!(status_after_block.unwrap().is_utxo_valid());
    assert!(status_after_block.unwrap().is_valid());

    // Selected chain tip should now be updated to this block
    assert_eq!(consensus.get_selected_chain_tip(), Some(expected_hash));
}

#[test]
fn test_deep_dag_reorganizations_and_blue_work() {
    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let mut now = jio_core::time::unix_now();

    // Build Branch A: Genesis -> A1 -> A2
    now += 1000;
    let block_a1 = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(1u64))
        .build();
    let a1_hash = consensus.validate_and_insert_block(block_a1).unwrap();
    assert_eq!(consensus.get_selected_chain_tip(), Some(a1_hash));

    now += 1000;
    let block_a2 = TestBlockBuilder::new(vec![a1_hash], 2)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(2u64))
        .build();
    let a2_hash = consensus.validate_and_insert_block(block_a2).unwrap();
    assert_eq!(consensus.get_selected_chain_tip(), Some(a2_hash));

    // Build Branch B: Genesis -> B1 -> B2 -> B3 (longer chain, higher work)
    now += 1000;
    let block_b1 = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(1u64))
        .build();
    let b1_hash = consensus.validate_and_insert_block(block_b1).unwrap();
    // Tip remains A2 since work is 2 vs 1
    assert_eq!(consensus.get_selected_chain_tip(), Some(a2_hash));

    now += 1000;
    let block_b2 = TestBlockBuilder::new(vec![b1_hash], 2)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(2u64))
        .build();
    let b2_hash = consensus.validate_and_insert_block(block_b2).unwrap();

    now += 1000;
    let block_b3 = TestBlockBuilder::new(vec![b2_hash], 3)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(3u64))
        .build();
    let b3_hash = consensus.validate_and_insert_block(block_b3).unwrap();

    // Reorganization to Branch B3 (work 3 > work 2)
    assert_eq!(consensus.get_selected_chain_tip(), Some(b3_hash));

    // Now extend Branch A: A2 -> A3 -> A4 (work 4 > work 3)
    now += 1000;
    let block_a3 = TestBlockBuilder::new(vec![a2_hash], 3)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(3u64))
        .build();
    let a3_hash = consensus.validate_and_insert_block(block_a3).unwrap();

    now += 1000;
    let block_a4 = TestBlockBuilder::new(vec![a3_hash], 4)
        .with_timestamp(now)
        .with_blue_work(Uint192::from(4u64))
        .build();
    let a4_hash = consensus.validate_and_insert_block(block_a4).unwrap();

    // Reorganization back to Branch A4
    assert_eq!(consensus.get_selected_chain_tip(), Some(a4_hash));
}

#[test]
fn test_orphan_block_dependency_management() {
    let dep_manager = BlockTaskDependencyManager::new();

    let parent_a = jio_hashes::Hash::from_le_u64([10, 0, 0, 0]);
    let parent_b = jio_hashes::Hash::from_le_u64([20, 0, 0, 0]);
    let child_1 = jio_hashes::Hash::from_le_u64([100, 0, 0, 0]);
    let child_2 = jio_hashes::Hash::from_le_u64([200, 0, 0, 0]);

    // child_1 depends on parent_a
    dep_manager.register_dependent(child_1, parent_a);

    // child_2 depends on both parent_a and parent_b
    dep_manager.register_dependent(child_2, parent_a);
    dep_manager.register_dependent(child_2, parent_b);

    // Satisfy parent_a: child_1 should be ready; child_2 still needs parent_b
    let ready_after_a = dep_manager.satisfy_dependency(&parent_a);
    assert_eq!(ready_after_a, vec![child_1]);

    // Satisfy parent_b: child_2 should now be ready
    let ready_after_b = dep_manager.satisfy_dependency(&parent_b);
    assert_eq!(ready_after_b, vec![child_2]);

    // Satisfying again yields empty list
    let ready_empty = dep_manager.satisfy_dependency(&parent_a);
    assert!(ready_empty.is_empty());
}
