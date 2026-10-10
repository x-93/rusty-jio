use crate::{
    hashing::HasherExtensions,
    tx::{TransactionOutpoint, UtxoEntry, VerifiableTransaction},
};
use jio_hashes::HasherBase;
use jio_muhash::MuHash;

struct UtxoWriter(Vec<u8>);

impl HasherBase for UtxoWriter {
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.0.extend_from_slice(data.as_ref());
        self
    }
}

pub trait MuHashExtensions {
    fn add_transaction(&mut self, tx: &impl VerifiableTransaction, block_daa_score: u64);
    fn add_utxo(&mut self, outpoint: &TransactionOutpoint, entry: &UtxoEntry);
}

impl MuHashExtensions for MuHash {
    fn add_transaction(&mut self, tx: &impl VerifiableTransaction, block_daa_score: u64) {
        let tx_id = tx.id();
        for (input, entry) in tx.populated_inputs() {
            let mut writer = UtxoWriter(Vec::with_capacity(70));
            write_utxo(&mut writer, entry, &input.previous_outpoint);
            self.remove_element(&writer.0);
        }
        for (i, output) in tx.outputs().iter().enumerate() {
            let outpoint = TransactionOutpoint::new(tx_id, i as u32);
            let entry = UtxoEntry::new(
                output.value,
                output.script_public_key.clone(),
                block_daa_score,
                tx.is_coinbase(),
            );
            self.add_utxo(&outpoint, &entry);
        }
    }

    fn add_utxo(&mut self, outpoint: &TransactionOutpoint, entry: &UtxoEntry) {
        let mut writer = UtxoWriter(Vec::with_capacity(70));
        write_utxo(&mut writer, entry, outpoint);
        self.add_element(&writer.0);
    }
}

fn write_utxo(writer: &mut impl HasherBase, entry: &UtxoEntry, outpoint: &TransactionOutpoint) {
    writer
        // Outpoint
        .update(outpoint.transaction_id)
        .update(outpoint.index.to_le_bytes())
        // Utxo entry
        .update(entry.block_daa_score.to_le_bytes())
        .update(entry.amount.to_le_bytes())
        .write_bool(entry.is_coinbase)
        .update(entry.script_public_key.version().to_le_bytes())
        .write_var_bytes(entry.script_public_key.script());
}
