use crate::model::candidate_tx::CandidateTransaction;
use jio_consensus::consensus::ctl::ConsensusCtl;
use jio_consensus_core::tx::Transaction;
use jio_consensus_core::utxo::UtxoEntry;
use jio_mining_errors::{MiningError, MiningResult};
use std::sync::Arc;

pub struct MempoolPopulator;

impl MempoolPopulator {
    pub fn populate_and_calculate_fee(
        consensus: &Arc<dyn ConsensusCtl>,
        tx: Arc<Transaction>,
    ) -> MiningResult<(CandidateTransaction, Vec<UtxoEntry>)> {
        let mut total_input_amount = 0u64;
        let mut populated_entries = Vec::with_capacity(tx.inputs.len());

        for input in tx.inputs.iter() {
            if let Some(entry) = consensus.get_utxo(&input.previous_outpoint) {
                total_input_amount = total_input_amount
                    .checked_add(entry.amount)
                    .ok_or_else(|| MiningError::TxRejected("input amount overflow".to_string()))?;
                populated_entries.push(entry);
            } else {
                return Err(MiningError::OrphanTransaction);
            }
        }

        let total_output_amount: u64 = tx.outputs.iter().map(|o| o.value).sum();
        if total_input_amount < total_output_amount {
            return Err(MiningError::TxRejected(format!(
                "total input amount {} is less than total output amount {}",
                total_input_amount, total_output_amount
            )));
        }

        let fee = total_input_amount - total_output_amount;
        let mass = tx.calc_mass();
        let candidate = CandidateTransaction::new(tx, fee, mass);

        Ok((candidate, populated_entries))
    }
}
