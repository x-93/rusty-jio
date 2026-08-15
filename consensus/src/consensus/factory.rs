use crate::consensus::ctl::ConsensusCtl;
use crate::consensus::services::ConsensusServices;
use crate::consensus::storage::ConsensusStorage;
use crate::model::stores::virtual_state::VirtualState;
use crate::pipeline::body_processor::BodyProcessor;
use crate::pipeline::header_processor::HeaderProcessor;
use crate::pipeline::virtual_processor::VirtualProcessor;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::config::params::Params;
use jio_consensus_core::errors::consensus::ConsensusResult;
use jio_consensus_core::header::Header;
use jio_consensus_core::tx::TransactionOutpoint;
use jio_consensus_core::utxo::{UtxoEntry, UtxoView};
use std::sync::Arc;

pub struct ConsensusInstance {
    pub storage: ConsensusStorage,
    pub services: ConsensusServices,
    pub header_processor: HeaderProcessor,
    pub body_processor: BodyProcessor,
    pub virtual_processor: VirtualProcessor,
    pub params: Params,
}

impl ConsensusInstance {
    pub fn new(params: Params) -> Arc<Self> {
        let storage = ConsensusStorage::new();
        let services = ConsensusServices::new(&storage, &params);

        let header_processor = HeaderProcessor::new(
            storage.header_store.clone(),
            storage.ghostdag_store.clone(),
            storage.relations_store.clone(),
            storage.statuses_store.clone(),
            services.ghostdag_manager.clone(),
            services.difficulty_manager.clone(),
            services.pmt_manager.clone(),
        );

        let body_processor = BodyProcessor::new(HeaderProcessor::new(
            storage.header_store.clone(),
            storage.ghostdag_store.clone(),
            storage.relations_store.clone(),
            storage.statuses_store.clone(),
            services.ghostdag_manager.clone(),
            services.difficulty_manager.clone(),
            services.pmt_manager.clone(),
        ));

        let virtual_processor = VirtualProcessor::new(
            storage.utxo_set_store.clone(),
            storage.virtual_state_store.clone(),
            storage.selected_chain_store.clone(),
            storage.ghostdag_store.clone(),
            storage.header_store.clone(),
            storage.statuses_store.clone(),
            storage.tips_store.clone(),
            services.ghostdag_manager.clone(),
            services.difficulty_manager.clone(),
            services.pmt_manager.clone(),
            params.coinbase_maturity,
        );

        let instance = Arc::new(Self {
            storage,
            services,
            header_processor,
            body_processor,
            virtual_processor,
            params: params.clone(),
        });

        // Initialize genesis block
        let genesis = params.genesis.to_block();
        let genesis_hash = genesis.hash();
        instance.storage.pruning_store.set_pruning_point(genesis_hash, 0);
        instance.storage.reachability_store.init_genesis(genesis_hash);
        let _ = instance.body_processor.process_body(&genesis);
        let _ = instance.virtual_processor.process_block(&genesis);

        instance
    }
}

impl ConsensusCtl for ConsensusInstance {
    fn validate_and_insert_block(&self, block: Block) -> ConsensusResult<BlockHash> {
        self.body_processor.process_body(&block)?;
        self.virtual_processor.process_block(&block)
    }

    fn validate_and_insert_header(&self, header: &Header) -> ConsensusResult<BlockHash> {
        self.header_processor.process_header(header)
    }

    fn get_virtual_state(&self) -> Option<Arc<VirtualState>> {
        self.storage.virtual_state_store.get()
    }

    fn get_header(&self, hash: &BlockHash) -> Option<Arc<Header>> {
        self.storage.header_store.get_header(hash)
    }

    fn get_status(&self, hash: &BlockHash) -> Option<BlockStatus> {
        self.storage.statuses_store.get(hash)
    }

    fn get_utxo(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.storage.utxo_set_store.get(outpoint)
    }

    fn get_selected_chain_tip(&self) -> Option<BlockHash> {
        self.storage.selected_chain_store.get_tip()
    }
}

pub struct ConsensusFactory;

impl ConsensusFactory {
    pub fn new_instance(params: Params) -> Arc<dyn ConsensusCtl> {
        ConsensusInstance::new(params)
    }
}
