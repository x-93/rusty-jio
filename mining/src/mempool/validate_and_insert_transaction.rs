use crate::mempool::check_transaction_standard::check_transaction_standard;
use crate::mempool::config::MempoolConfig;
use crate::mempool::populate::MempoolPopulator;
use crate::model::candidate_tx::CandidateTransaction;
use crate::model::owner_txs::OwnerTransactions;
use jio_consensus::consensus::ctl::ConsensusCtl;
use jio_consensus_core::tx::{Transaction, TransactionId};
use jio_mining_errors::{MiningError, MiningResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MempoolValidator;

impl MempoolValidator {
    pub fn validate_and_insert(
        consensus: &Arc<dyn ConsensusCtl>,
        config: &MempoolConfig,
        owner_txs: &OwnerTransactions,
        transactions: &Arc<RwLock<HashMap<TransactionId, CandidateTransaction>>>,
        orphans: &Arc<RwLock<HashMap<TransactionId, Arc<Transaction>>>>,
        tx: Arc<Transaction>,
    ) -> MiningResult<CandidateTransaction> {
        let tx_id = tx.id();

        // 1. Check standard transaction rules
        check_transaction_standard(&tx, config)?;

        // 2. Check if already in mempool
        if transactions.read().contains_key(&tx_id) {
            return Err(MiningError::TxAlreadyInMempool(tx_id.to_string()));
        }

        // 3. Check for double spends in mempool
        for input in &tx.inputs {
            if owner_txs.is_outpoint_spent(&input.previous_outpoint) {
                return Err(MiningError::TxRejected(format!(
                    "outpoint {:?} already spent in mempool",
                    input.previous_outpoint
                )));
            }
        }

        // 4. Try to populate inputs from consensus UTXO set
        let (candidate, _populated_entries) = match MempoolPopulator::populate_and_calculate_fee(consensus, tx.clone()) {
            Ok(res) => res,
            Err(MiningError::OrphanTransaction) => {
                // Add to orphan pool
                let mut orphan_pool = orphans.write();
                if orphan_pool.len() < config.maximum_orphan_transaction_count {
                    orphan_pool.insert(tx_id, tx);
                }
                return Err(MiningError::OrphanTransaction);
            }
            Err(e) => return Err(e),
        };

        // 5. Fee rate validation
        if candidate.fee_rate < config.minimum_relay_fee_rate {
            return Err(MiningError::FeeTooLow(
                candidate.fee,
                (config.minimum_relay_fee_rate * candidate.mass as f64) as u64,
            ));
        }

        // 6. Capacity check
        if transactions.read().len() >= config.maximum_transaction_count {
            return Err(MiningError::MempoolFull);
        }

        // 7. Insert transaction and record spent outpoints
        let outpoints = tx.inputs.iter().map(|i| i.previous_outpoint).collect();
        owner_txs.insert(tx_id, outpoints);
        transactions.write().insert(tx_id, candidate.clone());

        Ok(candidate)
    }
}
