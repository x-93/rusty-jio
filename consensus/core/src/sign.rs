use crate::hashing::sighash::calc_schnorr_signature_hash;
use crate::hashing::sighash_type::SigHashType;
use crate::tx::Transaction;
use crate::tx::script_public_key::ScriptPublicKey;
use jio_hashes::Hash;

/// Trait for transactions capable of generating standard sighashes for inputs.
pub trait SignableTransaction {
    fn calculate_sighash(
        &self,
        input_index: usize,
        hash_type: SigHashType,
        script_public_key: &ScriptPublicKey,
    ) -> Hash;
}

impl SignableTransaction for Transaction {
    fn calculate_sighash(
        &self,
        input_index: usize,
        hash_type: SigHashType,
        script_public_key: &ScriptPublicKey,
    ) -> Hash {
        calc_schnorr_signature_hash(self, input_index, hash_type, script_public_key)
    }
}
