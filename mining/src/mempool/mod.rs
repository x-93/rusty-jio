pub mod check_transaction_standard;
pub mod config;
pub mod populate;
pub mod validate_and_insert_transaction;

pub use check_transaction_standard::*;
pub use config::*;
pub use populate::*;
pub use validate_and_insert_transaction::*;

use crate::model::candidate_tx::CandidateTransaction;
use crate::model::owner_txs::OwnerTransactions;
use jio_consensus::consensus::ctl::ConsensusCtl;
use jio_consensus_core::tx::{Transaction, TransactionId};
use jio_mining_errors::MiningResult;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Mempool {
    config: MempoolConfig,
    owner_txs: OwnerTransactions,
    transactions: Arc<RwLock<HashMap<TransactionId, CandidateTransaction>>>,
    orphans: Arc<RwLock<HashMap<TransactionId, Arc<Transaction>>>>,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            owner_txs: OwnerTransactions::new(),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            orphans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert_transaction(
        &self,
        consensus: &Arc<dyn ConsensusCtl>,
        tx: Arc<Transaction>,
    ) -> MiningResult<CandidateTransaction> {
        MempoolValidator::validate_and_insert(
            consensus,
            &self.config,
            &self.owner_txs,
            &self.transactions,
            &self.orphans,
            tx,
        )
    }

    pub fn remove_transaction(&self, tx_id: &TransactionId) -> Option<CandidateTransaction> {
        self.owner_txs.remove(tx_id);
        self.transactions.write().remove(tx_id)
    }

    pub fn get_transaction(&self, tx_id: &TransactionId) -> Option<CandidateTransaction> {
        self.transactions.read().get(tx_id).cloned()
    }

    pub fn has_transaction(&self, tx_id: &TransactionId) -> bool {
        self.transactions.read().contains_key(tx_id)
    }

    pub fn get_all_candidates(&self) -> Vec<CandidateTransaction> {
        let mut candidates: Vec<_> = self.transactions.read().values().cloned().collect();
        // Sort highest fee rate first
        candidates.sort_by(|a, b| b.fee_rate.partial_cmp(&a.fee_rate).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    pub fn clean_committed_transactions(&self, committed: &[TransactionId]) {
        for tx_id in committed {
            self.remove_transaction(tx_id);
            self.orphans.write().remove(tx_id);
        }
    }

    pub fn len(&self) -> usize {
        self.transactions.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.read().is_empty()
    }

    pub fn orphan_count(&self) -> usize {
        self.orphans.read().len()
    }
}
