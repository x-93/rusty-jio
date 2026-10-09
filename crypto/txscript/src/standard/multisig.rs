//! Multi-signature script construction and evaluation routines.

use crate::script_builder::{OpCode, ScriptBuilder};
use crate::standard::{verify_schnorr_signature, TxScriptError};

/// Construct an M-of-N multisig script:
/// `OP_{m} <pubkey_1> <pubkey_2> ... <pubkey_n> OP_{n} OP_CHECKMULTISIG`
pub fn pay_to_multisig_script(required_sigs: usize, pubkeys: &[&[u8]]) -> Result<Vec<u8>, TxScriptError> {
    if required_sigs == 0 || required_sigs > pubkeys.len() || pubkeys.len() > 16 {
        return Err(TxScriptError::NonStandardScript);
    }

    let mut builder = ScriptBuilder::new();
    builder.add_i64(required_sigs as i64);

    for &pk in pubkeys {
        if pk.len() != 32 {
            return Err(TxScriptError::InvalidPublicKey);
        }
        builder.add_data(pk);
    }

    builder.add_i64(pubkeys.len() as i64);
    builder.add_op(OpCode::OpCheckMultiSig);

    Ok(builder.drain())
}

/// Verify an M-of-N multisig given a set of signatures, public keys, and sighash digest.
pub fn eval_multisig(required_sigs: usize, pubkeys: &[&[u8]], signatures: &[&[u8]], sighash: &[u8; 32]) -> bool {
    if signatures.len() < required_sigs {
        return false;
    }

    let mut sig_idx = 0;
    let mut pk_idx = 0;

    while sig_idx < signatures.len() && pk_idx < pubkeys.len() {
        let sig = signatures[sig_idx];
        let pk = pubkeys[pk_idx];

        if let Ok(true) = verify_schnorr_signature(pk, sig, sighash) {
            sig_idx += 1;
        }
        pk_idx += 1;
    }

    sig_idx >= required_sigs
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_hashes::Hash;
    use secp256k1::{Keypair, Message, Secp256k1};

    #[test]
    fn test_multisig_eval_2_of_3() {
        let secp = Secp256k1::new();
        let kp1 = Keypair::new(&secp, &mut rand::thread_rng());
        let kp2 = Keypair::new(&secp, &mut rand::thread_rng());
        let kp3 = Keypair::new(&secp, &mut rand::thread_rng());

        let pk1 = kp1.x_only_public_key().0.serialize();
        let pk2 = kp2.x_only_public_key().0.serialize();
        let pk3 = kp3.x_only_public_key().0.serialize();

        let sighash = Hash::from_u64_word(0x54321);
        let msg = Message::from_digest(sighash.as_bytes());

        let sig1 = secp.sign_schnorr(&msg, &kp1);
        let sig2 = secp.sign_schnorr(&msg, &kp2);

        let pks: &[&[u8]] = &[&pk1[..], &pk2[..], &pk3[..]];
        let sigs: &[&[u8]] = &[sig1.as_ref(), sig2.as_ref()];

        assert!(eval_multisig(2, pks, sigs, &sighash.as_bytes()));

        // 1 signature for 2-of-3 must fail
        assert!(!eval_multisig(2, pks, &[sig1.as_ref()], &sighash.as_bytes()));
    }
}
