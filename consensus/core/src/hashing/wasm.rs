use crate::header::Header;
use crate::tx::Transaction;
use jio_hashes::Hash;

pub fn wasm_header_hash(header: &Header) -> Hash {
    crate::hashing::header::header_hash(header)
}

pub fn wasm_tx_id(tx: &Transaction) -> Hash {
    crate::hashing::tx::tx_id(tx)
}
