use jio_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use jio_consensus_core::tx::{
    ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput,
};
use jio_consensus_core::utxo::UtxoEntry;

pub struct TransactionGenerator;

impl TransactionGenerator {
    pub fn create_unsigned_tx(
        utxos: &[(TransactionOutpoint, UtxoEntry)],
        destination: ScriptPublicKey,
        amount: u64,
        change_destination: ScriptPublicKey,
        fee: u64,
    ) -> Result<Transaction, String> {
        let total_needed = amount
            .checked_add(fee)
            .ok_or_else(|| "Amount + fee exceeds maximum u64 value".to_string())?;
        let mut total_in = 0u64;
        let mut inputs = Vec::new();

        for (op, entry) in utxos {
            inputs.push(TransactionInput {
                previous_outpoint: *op,
                signature_script: Vec::new(),
                sequence: 0,
                sig_op_count: 1,
            });
            total_in = total_in
                .checked_add(entry.amount)
                .ok_or_else(|| "Total inputs value overflow".to_string())?;
            if total_in >= total_needed {
                break;
            }
        }

        if total_in < total_needed {
            return Err(format!(
                "Insufficient funds: have {}, need {}",
                total_in, total_needed
            ));
        }

        let mut outputs = vec![TransactionOutput::new(amount, destination)];
        let change = total_in - total_needed;
        if change > 0 {
            outputs.push(TransactionOutput::new(change, change_destination));
        }

        Ok(Transaction {
            version: 0,
            inputs,
            outputs,
            lock_time: 0,
            subnetwork_id: SUBNETWORK_ID_NATIVE,
            gas: 0,
            payload: Vec::new(),
            mass: 0,
        })
    }
}
