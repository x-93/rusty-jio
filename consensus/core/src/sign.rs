use crate::hashing::sighash::calc_schnorr_signature_hash;
use crate::hashing::sighash_type::SigHashType;
use crate::tx::MutableTransaction;
use jio_hashes::Hash;

pub fn sign_transaction_input(
    mtx: &MutableTransaction,
    input_index: usize,
    hash_type: SigHashType,
) -> Hash {
    calc_schnorr_signature_hash(&mtx.tx, input_index, hash_type)
}
