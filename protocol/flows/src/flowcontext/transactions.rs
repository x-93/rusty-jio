use jio_consensus_core::tx::{Transaction, TransactionId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct TransactionsRelayPool {
    transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
}

impl TransactionsRelayPool {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, id: TransactionId, tx: Transaction) {
        self.transactions.write().insert(id, tx);
    }

    pub fn get(&self, id: &TransactionId) -> Option<Transaction> {
        self.transactions.read().get(id).cloned()
    }

    pub fn remove(&self, id: &TransactionId) -> Option<Transaction> {
        self.transactions.write().remove(id)
    }

    pub fn len(&self) -> usize {
        self.transactions.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.read().is_empty()
    }
}
