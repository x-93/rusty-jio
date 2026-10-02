//! Signing helpers.

use jio_hashes::Hash;

pub fn sign_message_hash(hash: &Hash, key: &[u8; 32]) -> [u8; 64] {
    let secp = secp256k1::Secp256k1::signing_only();
    let kp = secp256k1::Keypair::from_seckey_slice(&secp, key).expect("Valid seckey");
    let msg = secp256k1::Message::from_digest(hash.as_bytes());
    let sig = secp.sign_schnorr(&msg, &kp);
    *sig.as_ref()
}
