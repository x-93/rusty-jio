use jio_consensus_core::tx::{TransactionId, TransactionOutpoint};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct OwnerTransactions {
    // Maps spent outpoint to tx_id in mempool
    outpoint_to_tx: Arc<RwLock<HashMap<TransactionOutpoint, TransactionId>>>,
    // Maps tx_id to spent outpoints
    tx_to_outpoints: Arc<RwLock<HashMap<TransactionId, HashSet<TransactionOutpoint>>>>,
}

impl OwnerTransactions {
    pub fn new() -> Self {
        Self {
            outpoint_to_tx: Arc::new(RwLock::new(HashMap::new())),
            tx_to_outpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, tx_id: TransactionId, outpoints: Vec<TransactionOutpoint>) {
        let mut op_map = self.outpoint_to_tx.write();
        let mut tx_map = self.tx_to_outpoints.write();
        let mut set = HashSet::new();

        for op in outpoints {
            op_map.insert(op, tx_id);
            set.insert(op);
        }
        tx_map.insert(tx_id, set);
    }

    pub fn remove(&self, tx_id: &TransactionId) {
        let mut op_map = self.outpoint_to_tx.write();
        let mut tx_map = self.tx_to_outpoints.write();

        if let Some(outpoints) = tx_map.remove(tx_id) {
            for op in outpoints {
                op_map.remove(&op);
            }
        }
    }

    pub fn is_outpoint_spent(&self, outpoint: &TransactionOutpoint) -> bool {
        self.outpoint_to_tx.read().contains_key(outpoint)
    }

    pub fn get_spender(&self, outpoint: &TransactionOutpoint) -> Option<TransactionId> {
        self.outpoint_to_tx.read().get(outpoint).copied()
    }
}
