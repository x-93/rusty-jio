use crate::{hashing, tx::Transaction};
use jio_hashes::Hash;
use jio_merkle::calc_merkle_root;

pub fn calc_hash_merkle_root<'a>(
    txs: impl ExactSizeIterator<Item = &'a Transaction>,
    include_mass_field: bool,
) -> Hash {
    let hashes: Vec<Hash> = txs.map(|tx| hashing::tx::hash(tx, include_mass_field)).collect();
    calc_merkle_root(&hashes)
}
