//! Transaction signature hash calculation.

use super::super::tx::Transaction;
use super::sighash_type::SigHashType;
use jio_hashes::{Hash, HasherBase, TransactionSigningHash};

pub fn calc_sighash(tx: &Transaction, input_idx: usize, sig_hash_type: SigHashType) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    hasher.update(tx.id().as_bytes());
    hasher.update(&(input_idx as u64).to_le_bytes());
    hasher.update(&[sig_hash_type.to_u8()]);
    hasher.finalize()
}
