use crate::child_number::ChildNumber;
use hmac::{Hmac, Mac};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::Sha512;
use std::fmt;

type HmacSha512 = Hmac<Sha512>;

#[derive(Clone, PartialEq, Eq)]
pub struct ExtendedPrivateKey {
    pub secret_key: SecretKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
    pub parent_fingerprint: [u8; 4],
    pub child_number: ChildNumber,
}

impl fmt::Debug for ExtendedPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtendedPrivateKey")
            .field("depth", &self.depth)
            .field("child_number", &self.child_number)
            .finish()
    }
}

impl ExtendedPrivateKey {
    pub fn new_master(seed: &[u8]) -> Result<Self, String> {
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed").map_err(|e| e.to_string())?;
        mac.update(seed);
        let result = mac.finalize().into_bytes();

        let secret_key = SecretKey::from_slice(&result[0..32]).map_err(|e| e.to_string())?;
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&result[32..64]);

        Ok(Self {
            secret_key,
            chain_code,
            depth: 0,
            parent_fingerprint: [0u8; 4],
            child_number: ChildNumber(0),
        })
    }

    pub fn derive_child(&self, child_num: ChildNumber) -> Result<Self, String> {
        let secp = Secp256k1::new();
        let mut mac = HmacSha512::new_from_slice(&self.chain_code).map_err(|e| e.to_string())?;

        if child_num.is_hardened() {
            mac.update(&[0u8]);
            mac.update(&self.secret_key.secret_bytes());
        } else {
            let pubkey = PublicKey::from_secret_key(&secp, &self.secret_key);
            mac.update(&pubkey.serialize());
        }
        mac.update(&child_num.0.to_be_bytes());

        let result = mac.finalize().into_bytes();
        let derived_secret = SecretKey::from_slice(&result[0..32]).map_err(|e| e.to_string())?;
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&result[32..64]);

        let parent_pubkey = PublicKey::from_secret_key(&secp, &self.secret_key);
        let mut parent_fingerprint = [0u8; 4];
        parent_fingerprint.copy_from_slice(&parent_pubkey.serialize()[1..5]);

        Ok(Self {
            secret_key: derived_secret,
            chain_code,
            depth: self.depth.saturating_add(1),
            parent_fingerprint,
            child_number: child_num,
        })
    }

    pub fn to_public_key(&self) -> ExtendedPublicKey {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &self.secret_key);
        ExtendedPublicKey {
            public_key,
            chain_code: self.chain_code,
            depth: self.depth,
            parent_fingerprint: self.parent_fingerprint,
            child_number: self.child_number,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedPublicKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
    pub parent_fingerprint: [u8; 4],
    pub child_number: ChildNumber,
}
