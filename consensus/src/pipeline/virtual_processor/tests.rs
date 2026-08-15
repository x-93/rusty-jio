use crate::consensus::factory::ConsensusFactory;
use crate::pipeline::virtual_processor::test_block_builder::TestBlockBuilder;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::config::params::{Params, MAINNET_PARAMS};
use jio_consensus_core::constants::TX_VERSION;
use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::errors::consensus::ConsensusError;
use jio_consensus_core::errors::tx::TxRuleError;
use jio_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use jio_consensus_core::tx::{Transaction, TransactionInput, TransactionOutput, TransactionOutpoint};
use jio_hashes::Hash;
use jio_txscript::ScriptPublicKey;

#[test]
fn test_pow_validation_matrix() {
    use crate::pipeline::header_processor::post_pow_validation::validate_post_pow;
    use crate::processes::difficulty::check_hash_meets_difficulty;

    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let timestamp = jio_core::time::unix_now();

    // 1. Post-PoW validation on easy target (0x207f_ffff) succeeds
    let easy_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(timestamp)
        .with_bits(0x207f_ffff)
        .with_nonce(0)
        .build();
    assert!(validate_post_pow(&easy_block.header).is_ok());

    // 2. Post-PoW validation on high difficulty target without valid PoW fails
    let hard_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(timestamp + 5)
        .with_bits(0x1a00_ffff)
        .with_nonce(12345)
        .build();
    let err = validate_post_pow(&hard_block.header).unwrap_err();
    assert_eq!(err, BlockRuleError::InvalidTx("insufficient proof of work".to_string()));

    // 3. Difficulty check function matrix
    let easy_bits = 0x207f_ffff;
    let low_hash = Hash::from_bytes([0x01; 32]);
    assert!(check_hash_meets_difficulty(&low_hash, easy_bits).unwrap());

    let high_hash = Hash::from_bytes([0xff; 32]);
    assert!(!check_hash_meets_difficulty(&high_hash, 0x1a00_ffff).unwrap());

    // 4. Consensus insertion with valid bits succeeds
    let hash = consensus.validate_and_insert_block(easy_block);
    assert!(hash.is_ok(), "easy block with valid expected bits should pass");

    // 5. Consensus insertion with unexpected bits fails pre-ghostdag check
    let bad_bits_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(timestamp + 10)
        .with_bits(0x1a00_ffff)
        .build();
    let err = consensus.validate_and_insert_block(bad_bits_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::InvalidTx(msg)) => {
            assert!(msg.contains("unexpected difficulty bits"));
        }
        other => panic!("expected unexpected difficulty bits error, got {other:?}"),
    }
}

#[test]
fn test_block_timestamp_future_and_pmt_rejection() {
    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let now = jio_core::time::unix_now();

    // 1. Timestamp in distant future (> 60 seconds drift)
    let future_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now + 120_000)
        .build();
    let err = consensus.validate_and_insert_block(future_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::TimeTooNew(_)) => {}
        other => panic!("expected TimeTooNew error, got {other:?}"),
    }

    // 2. Insert valid block 1
    let block1 = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .build();
    let block1_hash = consensus.validate_and_insert_block(block1).unwrap();

    // 3. Block 2 with timestamp <= past median time of parent
    let past_block = TestBlockBuilder::new(vec![block1_hash], 2)
        .with_timestamp(now - 10_000)
        .build();
    let err = consensus.validate_and_insert_block(past_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::TimeTooOld(_, _)) => {}
        other => panic!("expected TimeTooOld error, got {other:?}"),
    }
}

#[test]
fn test_header_validation_rules() {
    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let now = jio_core::time::unix_now();

    // 1. Bad block version
    let bad_version_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_version(99)
        .with_timestamp(now)
        .build();
    let err = consensus.validate_and_insert_block(bad_version_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::BadVersion(99)) => {}
        other => panic!("expected BadVersion error, got {other:?}"),
    }

    // 2. Duplicate parents
    let dup_parents_block = TestBlockBuilder::new(vec![genesis_hash, genesis_hash], 1)
        .with_timestamp(now)
        .build();
    let err = consensus.validate_and_insert_block(dup_parents_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::DuplicateParent) => {}
        other => panic!("expected DuplicateParent error, got {other:?}"),
    }

    // 3. Too many parents
    let mut excessive_parents = Vec::new();
    for i in 0..=32 {
        excessive_parents.push(Hash::from_le_u64([i + 1, 0, 0, 0]));
    }
    let too_many_parents_block = TestBlockBuilder::new(excessive_parents, 1)
        .with_timestamp(now)
        .build();
    let err = consensus.validate_and_insert_block(too_many_parents_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::TooManyParents(_, _)) => {}
        other => panic!("expected TooManyParents error, got {other:?}"),
    }
}

#[test]
fn test_block_body_isolation_and_context_rejection() {
    let params = Params::devnet();
    let consensus = ConsensusFactory::new_instance(params);
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let now = jio_core::time::unix_now();

    // 1. Block with empty transactions
    let empty_tx_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .omit_coinbase()
        .build();
    let err = consensus.validate_and_insert_block(empty_tx_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::InvalidTx(msg)) => {
            assert!(msg.contains("no transactions"));
        }
        other => panic!("expected no transactions error, got {other:?}"),
    }

    // 2. First transaction is non-coinbase
    let regular_tx = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput::new(TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0), vec![], 0, 0)],
        outputs: vec![TransactionOutput::new(100, ScriptPublicKey::default())],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_NATIVE,
        gas: 0,
        payload: vec![],
        mass: 10,
    };
    let non_cb_first = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .omit_coinbase()
        .with_tx(regular_tx.clone())
        .build();
    let err = consensus.validate_and_insert_block(non_cb_first).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::InvalidTx(msg)) => {
            assert!(msg.contains("first transaction must be coinbase"));
        }
        other => panic!("expected first tx coinbase error, got {other:?}"),
    }

    // 3. Block with multiple coinbase transactions
    let second_coinbase = crate::processes::coinbase::create_coinbase_transaction(1, 500, ScriptPublicKey::default());
    let multi_cb_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_tx(second_coinbase)
        .build();
    let err = consensus.validate_and_insert_block(multi_cb_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::InvalidTx(msg)) => {
            assert!(msg.contains("cannot be coinbase"));
        }
        other => panic!("expected multi coinbase error, got {other:?}"),
    }

    // 4. Block exceeding MAX_BLOCK_MASS
    let mut heavy_tx = regular_tx.clone();
    heavy_tx.mass = MAINNET_PARAMS.max_block_mass + 1;
    let heavy_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_tx(heavy_tx)
        .build();
    let err = consensus.validate_and_insert_block(heavy_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::ExceedsMassLimit(_, _)) => {}
        other => panic!("expected ExceedsMassLimit error, got {other:?}"),
    }

    // 5. Mismatched Merkle Root
    let bad_merkle_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_override_merkle_root(Hash::from_le_u64([0xdeadbeef, 0, 0, 0]))
        .build();
    let err = consensus.validate_and_insert_block(bad_merkle_block).unwrap_err();
    match err {
        ConsensusError::BlockRule(BlockRuleError::BadMerkleRoot { .. }) => {}
        other => panic!("expected BadMerkleRoot error, got {other:?}"),
    }
}

#[test]
fn test_coinbase_maturity_and_utxo_lifecycle() {
    let mut params = Params::devnet();
    params.coinbase_maturity = 5; // Set short maturity for test
    let consensus = ConsensusFactory::new_instance(params.clone());
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let mut now = jio_core::time::unix_now();

    // 1. Genesis coinbase output should be registered in UTXO set
    let genesis_block = params.genesis.to_block();
    let genesis_cb_id = genesis_block.transactions[0].id();
    let genesis_outpoint = TransactionOutpoint::new(genesis_cb_id, 0);
    let genesis_utxo = consensus.get_utxo(&genesis_outpoint);
    assert!(genesis_utxo.is_some(), "genesis coinbase UTXO must exist");
    assert!(genesis_utxo.unwrap().is_coinbase);

    // 2. Block 1: Mined at DAA score 1. Attempts to spend genesis coinbase before maturity (maturity = 5)
    let spend_tx = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput::new(genesis_outpoint, vec![], 0, 0)],
        outputs: vec![TransactionOutput::new(genesis_block.transactions[0].outputs[0].value, ScriptPublicKey::new(0, vec![9, 9, 9]))],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_NATIVE,
        gas: 0,
        payload: vec![],
        mass: 10,
    };

    let immature_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_tx(spend_tx.clone())
        .build();

    let err = consensus.validate_and_insert_block(immature_block).unwrap_err();
    match err {
        ConsensusError::TxRule(TxRuleError::ScriptFailed(msg)) => {
            assert!(msg.contains("coinbase maturity not reached"), "expected maturity error, got {msg}");
        }
        other => panic!("expected coinbase maturity not reached error, got {other:?}"),
    }

    // 3. Advance DAG by 5 blocks to satisfy maturity
    let mut parent = genesis_hash;
    for i in 1..=5u64 {
        now += 1000;
        let blk = TestBlockBuilder::new(vec![parent], i)
            .with_timestamp(now)
            .build();
        parent = consensus.validate_and_insert_block(blk).expect("mining block to mature coinbase");
    }

    // 4. Now at DAA score 6 >= 0 + 5 maturity: spending genesis coinbase should succeed!
    now += 1000;
    let mature_block = TestBlockBuilder::new(vec![parent], 6)
        .with_timestamp(now)
        .with_tx(spend_tx.clone())
        .build();
    let blk6_hash = consensus.validate_and_insert_block(mature_block).expect("spend at maturity must succeed");

    assert_eq!(consensus.get_status(&blk6_hash), Some(BlockStatus::StatusUTXOValid));

    // Spent output should be gone from UTXO set
    assert!(consensus.get_utxo(&genesis_outpoint).is_none(), "spent UTXO must be removed");
    // New output should exist
    let new_outpoint = TransactionOutpoint::new(spend_tx.id(), 0);
    assert!(consensus.get_utxo(&new_outpoint).is_some(), "new UTXO must be created");
}

#[test]
fn test_utxo_double_spend_and_negative_fee_prevention() {
    let mut params = Params::devnet();
    params.coinbase_maturity = 0; // Immediate maturity for simple testing
    let consensus = ConsensusFactory::new_instance(params.clone());
    let genesis_hash = consensus.get_virtual_state().unwrap().parents[0];
    let genesis_block = params.genesis.to_block();
    let genesis_cb_id = genesis_block.transactions[0].id();
    let genesis_outpoint = TransactionOutpoint::new(genesis_cb_id, 0);
    let genesis_val = genesis_block.transactions[0].outputs[0].value;
    let now = jio_core::time::unix_now();

    // 1. Intra-block double spend: Block with 2 transactions spending the same genesis outpoint
    let tx1 = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput::new(genesis_outpoint, vec![], 0, 0)],
        outputs: vec![TransactionOutput::new(genesis_val / 2, ScriptPublicKey::new(0, vec![1]))],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_NATIVE,
        gas: 0,
        payload: vec![],
        mass: 10,
    };
    let tx2 = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput::new(genesis_outpoint, vec![], 0, 0)],
        outputs: vec![TransactionOutput::new(genesis_val / 2, ScriptPublicKey::new(0, vec![2]))],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_NATIVE,
        gas: 0,
        payload: vec![],
        mass: 10,
    };

    let double_spend_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_tx(tx1.clone())
        .with_tx(tx2)
        .build();
    let err = consensus.validate_and_insert_block(double_spend_block).unwrap_err();
    match err {
        ConsensusError::TxRule(jio_consensus_core::errors::tx::TxRuleError::ScriptFailed(msg)) => {
            assert!(msg.contains("double spend detected"), "expected double spend error, got {msg}");
        }
        other => panic!("expected double spend error, got {other:?}"),
    }

    // 2. Negative fee transaction: Transaction where outputs > inputs
    let negative_fee_tx = Transaction {
        version: TX_VERSION,
        inputs: vec![TransactionInput::new(genesis_outpoint, vec![], 0, 0)],
        outputs: vec![TransactionOutput::new(genesis_val + 10_000, ScriptPublicKey::new(0, vec![1]))],
        lock_time: 0,
        subnetwork_id: SUBNETWORK_ID_NATIVE,
        gas: 0,
        payload: vec![],
        mass: 10,
    };
    let negative_fee_block = TestBlockBuilder::new(vec![genesis_hash], 1)
        .with_timestamp(now)
        .with_tx(negative_fee_tx)
        .build();
    let err = consensus.validate_and_insert_block(negative_fee_block).unwrap_err();
    match err {
        ConsensusError::TxRule(jio_consensus_core::errors::tx::TxRuleError::NegativeFee(fee)) => {
            assert_eq!(fee, -10_000);
        }
        other => panic!("expected NegativeFee error, got {other:?}"),
    }
}

#[test]
fn test_reachability_interval_and_dag_ancestor() {
    use crate::model::services::reachability::{MTReachabilityService, ReachabilityService};
    use crate::model::stores::reachability::ReachabilityStore;
    use crate::model::stores::relations::RelationsStore;
    use parking_lot::RwLock;
    use std::sync::Arc;

    let relations = RelationsStore::new();
    let reachability_store = ReachabilityStore::new();
    let reachability =
        MTReachabilityService::new(Arc::new(RwLock::new(reachability_store.clone())));

    let genesis = Hash::from_bytes([1u8; 32]);
    let block_a = Hash::from_bytes([2u8; 32]);
    let block_b = Hash::from_bytes([3u8; 32]);
    let block_c = Hash::from_bytes([4u8; 32]);

    // Genesis setup
    reachability_store.init_genesis(genesis);

    // Block A is child of Genesis
    relations.insert(block_a, vec![genesis]);
    reachability_store.add_block(block_a, genesis);

    // Block B is child of Block A
    relations.insert(block_b, vec![block_a]);
    reachability_store.add_block(block_b, block_a);

    // Block C merges Genesis and Block B
    relations.insert(block_c, vec![genesis, block_b]);
    reachability_store.add_block(block_c, block_b);

    // Assert ancestry
    assert!(reachability.is_dag_ancestor_of(genesis, block_a));
    assert!(reachability.is_dag_ancestor_of(genesis, block_b));
    assert!(reachability.is_dag_ancestor_of(genesis, block_c));
    assert!(reachability.is_dag_ancestor_of(block_a, block_b));
    assert!(reachability.is_dag_ancestor_of(block_a, block_c));
    assert!(reachability.is_dag_ancestor_of(block_b, block_c));

    // Non-ancestor
    assert!(!reachability.is_dag_ancestor_of(block_c, genesis));
    assert!(!reachability.is_dag_ancestor_of(block_b, block_a));
}

#[test]
fn test_pruning_manager_and_proof_lifecycle() {
    use crate::model::stores::ghostdag::GhostdagStore;
    use crate::model::stores::headers::HeaderStore;
    use crate::model::stores::pruning::PruningStore;
    use crate::processes::pruning::PruningManager;
    use crate::processes::pruning_proof::PruningProofManager;

    let params = Params::devnet();
    let pruning_store = PruningStore::new();
    let header_store = HeaderStore::new();
    let ghostdag_store = GhostdagStore::new();

    let pruning_mgr = PruningManager::new(
        params.clone(),
        pruning_store.clone(),
        header_store.clone(),
        ghostdag_store.clone(),
    );

    let proof_mgr = PruningProofManager::new(
        params.clone(),
        header_store.clone(),
        pruning_store.clone(),
    );

    // 1. Initial genesis state
    let genesis = params.genesis.to_block();
    header_store.insert(genesis.hash(), std::sync::Arc::new(genesis.header.clone()));
    pruning_store.set_pruning_point(genesis.hash(), 0);

    assert_eq!(pruning_mgr.pruning_point(), Some(genesis.hash()));
    assert!(!pruning_mgr.is_pruned(&genesis.hash()));

    // 2. Build pruning proof at genesis
    let proof = proof_mgr.build_pruning_point_proof().unwrap();
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0].len(), 1);
    assert_eq!(proof[0][0].hash, genesis.hash());

    // 3. Validate pruning proof
    let validated_point = proof_mgr.validate_pruning_point_proof(&proof).unwrap();
    assert_eq!(validated_point, genesis.hash());
}

#[test]
fn test_acceptance_data_store_lifecycle() {
    use crate::model::stores::acceptance_data::AcceptanceDataStore;
    use jio_consensus_core::acceptance_data::{AcceptedTxEntry, MergesetBlockAcceptanceData};

    let store = AcceptanceDataStore::new();
    let block_hash = Hash::from_bytes([0xaa; 32]);
    let tx_id = Hash::from_bytes([0xbb; 32]);

    let entry = AcceptedTxEntry {
        transaction_id: tx_id,
        index_within_block: 0,
    };
    let data = vec![MergesetBlockAcceptanceData {
        block_hash,
        accepted_transactions: vec![entry],
    }];

    store.insert(block_hash, std::sync::Arc::new(data));
    assert!(store.has(&block_hash));

    let fetched = store.get(&block_hash).unwrap();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].block_hash, block_hash);
    assert_eq!(fetched[0].accepted_transactions[0].transaction_id, tx_id);
}

