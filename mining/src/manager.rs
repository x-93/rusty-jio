use crate::block_template::builder::{BlockTemplate, BlockTemplateBuilder};
use crate::mempool::config::MempoolConfig;
use crate::mempool::Mempool;
use crate::model::candidate_tx::CandidateTransaction;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::errors::consensus::ConsensusResult;
use jio_consensus_core::tx::{ScriptPublicKey, Transaction};
use jio_consensusmanager::ConsensusManager;
use jio_mining_errors::MiningResult;
use std::sync::Arc;

#[derive(Clone)]
pub struct MiningManager {
    consensus_manager: ConsensusManager,
    mempool: Mempool,
}

impl MiningManager {
    pub fn new(consensus_manager: ConsensusManager, config: MempoolConfig) -> Self {
        Self {
            consensus_manager,
            mempool: Mempool::new(config),
        }
    }

    pub fn get_block_template(
        &self,
        payee_script_public_key: ScriptPublicKey,
        extra_data: Vec<u8>,
    ) -> MiningResult<BlockTemplate> {
        let session = self.consensus_manager.session();
        BlockTemplateBuilder::build_block_template(
            &session,
            &self.mempool,
            payee_script_public_key,
            extra_data,
        )
    }

    pub fn validate_and_insert_transaction(
        &self,
        tx: Arc<Transaction>,
    ) -> MiningResult<CandidateTransaction> {
        let session = self.consensus_manager.session();
        self.mempool.insert_transaction(&session, tx)
    }

    pub fn submit_block(&self, block: Block) -> ConsensusResult<BlockHash> {
        let session = self.consensus_manager.session();
        let tx_ids: Vec<_> = block.transactions.iter().map(|tx| tx.id()).collect();
        let hash = session.validate_and_insert_block(block)?;
        self.mempool.clean_committed_transactions(&tx_ids);
        Ok(hash)
    }

    pub fn mempool(&self) -> &Mempool {
        &self.mempool
    }
}
