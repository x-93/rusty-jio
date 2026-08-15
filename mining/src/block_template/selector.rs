use crate::model::candidate_tx::CandidateTransaction;
use jio_consensus_core::tx::Transaction;
use std::collections::HashSet;
use std::sync::Arc;

pub struct TransactionSelector;

impl TransactionSelector {
    pub fn select_transactions(
        candidates: Vec<CandidateTransaction>,
        max_block_mass: u64,
    ) -> (Vec<Arc<Transaction>>, u64, u64) {
        let mut selected = Vec::new();
        let mut total_fees = 0u64;
        let mut total_mass = 0u64;
        let mut spent_outpoints = HashSet::new();

        let limit = max_block_mass;

        for candidate in candidates {
            if total_mass + candidate.mass > limit {
                continue;
            }

            // Verify no conflicting inputs within the candidate batch
            let has_conflict = candidate
                .tx
                .inputs
                .iter()
                .any(|input| spent_outpoints.contains(&input.previous_outpoint));

            if !has_conflict {
                for input in &candidate.tx.inputs {
                    spent_outpoints.insert(input.previous_outpoint);
                }
                total_fees += candidate.fee;
                total_mass += candidate.mass;
                selected.push(candidate.tx.clone());
            }
        }

        (selected, total_fees, total_mass)
    }
}
