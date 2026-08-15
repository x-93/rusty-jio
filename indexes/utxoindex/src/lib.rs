pub mod core;
pub mod index;
pub mod model;
pub mod stores;
pub mod update;

pub use core::*;
pub use index::*;
pub use model::*;
pub use stores::*;
pub use update::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
    use jio_consensus_core::utxo::{UtxoDiff, UtxoEntry};
    use jio_hashes::Hash;

    #[test]
    fn test_utxo_index_flow() {
        let index = UtxoIndex::new();
        let spk = ScriptPublicKey::from_vec(0, vec![1, 2, 3, 4]);
        let outpoint = TransactionOutpoint::new(Hash::from_bytes([7u8; 32]), 0);
        let entry = UtxoEntry::new(1_000_000, spk.clone(), 100, false);

        let mut diff = UtxoDiff::default();
        diff.to_add.insert(outpoint, entry.clone());

        index.update(&diff);

        let utxos = index.get_utxos_by_script_public_key(&spk).expect("utxos exist");
        assert_eq!(utxos.len(), 1);
        assert_eq!(index.get_circulating_supply(), 1_000_000);

        // Remove
        let mut remove_diff = UtxoDiff::default();
        remove_diff.to_remove.insert(outpoint, entry);
        index.update(&remove_diff);

        assert_eq!(index.get_utxos_by_script_public_key(&spk), None);
        assert_eq!(index.get_circulating_supply(), 0);
    }
}
