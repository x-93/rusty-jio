pub mod services;
pub mod storage;

use crate::model::stores::ghostdag::GhostdagData;
use crate::model::stores::headers::HeaderStoreReader;
use crate::pipeline::body_processor::BodyProcessor;
use crate::pipeline::header_processor::HeaderProcessor;
use crate::pipeline::virtual_processor::VirtualStateProcessor;
use consensus_core::block::Block;
use consensus_core::config::params::Params;
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::mass::TxMassCalculator;
use jio_database::prelude::{BatchDbWriter, DB};
use services::ConsensusServices;
use std::sync::Arc;
use storage::ConsensusStorage;

pub struct Consensus {
    pub storage: ConsensusStorage,
    pub services: ConsensusServices,
    pub header_processor: HeaderProcessor<
        crate::model::stores::headers::DbHeaderStore,
        crate::model::stores::relations::DbRelationsStore,
        crate::model::stores::reachability::DbReachabilityStore,
        crate::model::stores::ghostdag::DbGhostdagStore,
        crate::model::stores::statuses::DbStatusesStore,
    >,
    pub body_processor: BodyProcessor<crate::model::stores::statuses::DbStatusesStore>,
    pub virtual_processor: VirtualStateProcessor<
        crate::model::stores::utxo_diffs::DbUtxoDiffsStore,
        crate::model::stores::statuses::DbStatusesStore,
    >,
    pub params: Params,
}

impl Consensus {
    pub fn new(db: Arc<DB>, params: Params) -> Self {
        let storage = ConsensusStorage::new(db.clone(), 10_000);
        let services = ConsensusServices::new(
            &storage,
            params.ghostdag_k,
            params.target_time_per_block,
            params.max_difficulty_target,
        );

        let mass_calculator = TxMassCalculator::new(1, 10, 1000);

        let header_processor = HeaderProcessor::new(
            storage.headers.clone(),
            storage.relations.clone(),
            services.reachability_manager.clone(),
            services.ghostdag_manager.clone(),
            storage.ghostdag.clone(),
            storage.statuses.clone(),
            services.difficulty_manager.clone(),
            params.timestamp_deviation_tolerance * 1000,
            params.genesis.header.bits,
            params.genesis.header.hash,
        );

        let body_processor = BodyProcessor::new(
            storage.statuses.clone(),
            mass_calculator,
            params.max_block_mass,
            150,
        );

        let virtual_processor = VirtualStateProcessor::new(
            storage.utxo_diffs.clone(),
            storage.statuses.clone(),
            100, // Legacy coinbase maturity
        );

        Self {
            storage,
            services,
            header_processor,
            body_processor,
            virtual_processor,
            params,
        }
    }

    /// Initializes consensus storage with the Genesis block if not already initialized.
    pub fn init(&self) -> BlockProcessResult<()> {
        if !self.storage.headers.has(self.params.genesis.header.hash) {
            let mut writer = BatchDbWriter::new(&self.storage.db);
            let genesis_block = self.params.genesis.to_block();
            self.header_processor
                .process_header(&mut writer, genesis_block.header.clone())?;
            self.body_processor
                .process_body(&mut writer, &genesis_block)?;
            self.virtual_processor
                .process_block(&mut writer, &genesis_block)
                .map_err(|_| RuleError::KnownInvalid(self.params.genesis.header.hash))?;
            writer
                .commit()
                .map_err(|_| RuleError::KnownInvalid(self.params.genesis.header.hash))?;
        }
        Ok(())
    }

    /// Ingests, validates, and commits an incoming block through the entire consensus pipeline.
    pub fn validate_and_insert_block(&self, block: Block) -> BlockProcessResult<Arc<GhostdagData>> {
        let mut writer = BatchDbWriter::new(&self.storage.db);

        // Stage 1: Header Processing (PoW, PMT, DAA, Reachability, GHOSTDAG election)
        let ghostdag_data = self
            .header_processor
            .process_header(&mut writer, block.header.clone())?;

        // Stage 2: Body Processing (isolated tx validation, Merkle root verification)
        self.body_processor.process_body(&mut writer, &block)?;

        // Stage 3: Virtual State Processing (UTXO diff calculation, MuHash commitment update)
        self.virtual_processor
            .process_block(&mut writer, &block)
            .map_err(|_| RuleError::KnownInvalid(block.header.hash))?;

        // Stage 4: Atomic Commit to RocksDB
        writer
            .commit()
            .map_err(|_| RuleError::KnownInvalid(block.header.hash))?;

        Ok(ghostdag_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::BlueWorkType;
    use consensus_core::config::params::default_mainnet_params;
    use consensus_core::header::Header;
    use consensus_core::merkle::calc_block_hash_merkle_root;
    use consensus_core::subnets::SubnetworkId;
    use consensus_core::tx::{ScriptPublicKey, Transaction, TransactionOutput};
    use jio_database::prelude::DB;
    use jio_hashes::Hash;
    use jio_math::Uint256;
    use tempfile::TempDir;

    #[test]
    fn test_full_consensus_pipeline_integration() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(DB::open_default(temp_dir.path()).unwrap());

        let mut params = default_mainnet_params();
        params.genesis.header.bits = 0x207fffff; // Easy target for test mining
        params.genesis.header.finalize();

        let consensus = Consensus::new(db, params.clone());

        // 1. Initialize Genesis
        consensus.init().unwrap();
        assert!(consensus.storage.headers.has(params.genesis.header.hash));

        // 2. Mine and submit block 1 on top of Genesis
        let genesis_hash = params.genesis.header.hash;
        let true_spk = ScriptPublicKey::from_vec(0, vec![jio_txscript::op::OP_TRUE]);
        let cb_tx = Transaction::new(
            1,
            vec![],
            vec![TransactionOutput::new(500_000_000, true_spk)],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![],
        );

        let txs = vec![cb_tx];
        let merkle_root = calc_block_hash_merkle_root(&txs);

        let mut header1 = Header::new_finalized(
            1,
            vec![vec![genesis_hash]],
            merkle_root,
            Hash::default(),
            Hash::default(),
            params.genesis.header.timestamp + 1000,
            params.genesis.header.bits,
            0,
            1,
            BlueWorkType::ZERO,
            1,
            Hash::default(),
        );

        // Find valid PoW nonce
        let pre_pow = header1.pre_pow_hash();
        let target = Uint256::from_compact_target_bits(header1.bits);
        for n in 0..10_000 {
            if jio_pow::check_proof_of_work(pre_pow, header1.timestamp, n, target).is_ok() {
                header1.nonce = n;
                break;
            }
        }
        header1.finalize();

        let block1 = Block::new(header1, txs);
        let gd1 = consensus.validate_and_insert_block(block1).unwrap();

        assert_eq!(gd1.selected_parent, genesis_hash);
        assert_eq!(gd1.blue_score, 1);
        assert_eq!(consensus.virtual_processor.utxo_set_len(), 1); // Block 1 coinbase output
    }
}
