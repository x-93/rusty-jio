use crate::hashing::sighash::calc_schnorr_signature_hash;
use crate::hashing::sighash_type::SigHashType;
use crate::tx::Transaction;
use secp256k1::{Keypair, Message, Secp256k1};

pub fn sign_transaction_input(
    tx: &mut Transaction,
    input_index: usize,
    keypair: &Keypair,
    hash_type: SigHashType,
) -> Result<(), secp256k1::Error> {
    let hash = calc_schnorr_signature_hash(tx, input_index, hash_type);
    let msg = Message::from_digest(*hash.as_bytes());
    let secp = Secp256k1::signing_only();
    let sig = secp.sign_schnorr(&msg, keypair);

    let mut sig_script = Vec::with_capacity(65);
    sig_script.extend_from_slice(sig.as_ref());
    sig_script.push(hash_type.to_u8());
    tx.inputs[input_index].signature_script = sig_script;
    Ok(())
}