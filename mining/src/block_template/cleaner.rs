use crate::mempool::Mempool;
use jio_consensus_core::tx::TransactionId;

pub struct TransactionCleaner;

impl TransactionCleaner {
    pub fn clean_accepted_transactions(mempool: &Mempool, accepted_tx_ids: &[TransactionId]) {
        mempool.clean_committed_transactions(accepted_tx_ids);
    }
}
