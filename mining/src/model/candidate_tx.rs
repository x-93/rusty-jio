use jio_consensus_core::tx::{Transaction, TransactionId};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CandidateTransaction {
    pub tx: Arc<Transaction>,
    pub fee: u64,
    pub mass: u64,
    pub fee_rate: f64,
    pub insertion_time: u64,
}

impl CandidateTransaction {
    pub fn new(tx: Arc<Transaction>, fee: u64, mass: u64) -> Self {
        let effective_mass = mass.max(1);
        let fee_rate = (fee as f64) / (effective_mass as f64);
        Self {
            tx,
            fee,
            mass,
            fee_rate,
            insertion_time: jio_core::time::unix_now(),
        }
    }

    pub fn id(&self) -> TransactionId {
        self.tx.id()
    }
}
