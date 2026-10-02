//! Transaction hashing helpers.

use super::super::tx::Transaction;
use jio_hashes::Hash;

pub fn id(tx: &Transaction) -> Hash {
    tx.id()
}

pub fn hash(tx: &Transaction) -> Hash {
    tx.hash()
}
