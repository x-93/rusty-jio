// standard.rs
//
// Standard script templates for rusty-jio:
// - Pay-to-Pubkey (P2PK) with Schnorr (version 0)
// - Pay-to-Pubkey (P2PK) with ECDSA (version 1)
// - Pay-to-Script-Hash (P2SH)

use crate::opcodes::codes::*;
use crate::script_builder::ScriptBuilder;
use borsh::{BorshDeserialize, BorshSerialize};
use jio_addresses::{Address, Prefix, Version};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::io::{Read, Write};

/// Standard locking script attached to a UTXO (Transaction Output).
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: SmallVec<[u8; 36]>,
}

impl ScriptPublicKey {
    #[inline(always)]
    pub fn new(version: u16, script: SmallVec<[u8; 36]>) -> Self {
        Self { version, script }
    }

    #[inline(always)]
    pub fn from_vec(version: u16, script: Vec<u8>) -> Self {
        Self {
            version,
            script: SmallVec::from_vec(script),
        }
    }
}

// Manual Borsh implementation for ScriptPublicKey
impl BorshSerialize for ScriptPublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.version, writer)?;
        BorshSerialize::serialize(&(self.script.len() as u32), writer)?;
        writer.write_all(&self.script)?;
        Ok(())
    }
}

impl BorshDeserialize for ScriptPublicKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let version = u16::deserialize_reader(reader)?;
        let len = u32::deserialize_reader(reader)? as usize;
        let mut script = vec![0u8; len];
        reader.read_exact(&mut script)?;
        Ok(ScriptPublicKey::from_vec(version, script))
    }
}

/// Generates a standard Schnorr Pay-to-Pubkey (P2PK) script:
/// `<32-byte Schnorr X-only Pubkey> OP_CHECKSIG`
pub fn pay_to_pubkey_script(pubkey: &[u8; 32]) -> ScriptPublicKey {
    let script = ScriptBuilder::new()
        .add_data(pubkey)
        .add_op(OP_CHECKSIG)
        .to_smallvec();
    ScriptPublicKey::new(0, script)
}

/// Generates an ECDSA Pay-to-Pubkey (P2PK) script:
/// `<33-byte compressed Secp256k1 Pubkey> OP_CHECKSIG_ECDSA`
pub fn pay_to_pubkey_ecdsa_script(pubkey: &[u8; 33]) -> ScriptPublicKey {
    let script = ScriptBuilder::new()
        .add_data(pubkey)
        .add_op(OP_CHECKSIG_ECDSA)
        .to_smallvec();
    ScriptPublicKey::new(0, script)
}

/// Generates a Pay-to-Script-Hash (P2SH) script:
/// `OP_BLAKE2B <32-byte Script Hash> OP_EQUAL`
pub fn pay_to_script_hash_script(script_hash: &[u8; 32]) -> ScriptPublicKey {
    let script = ScriptBuilder::new()
        .add_op(OP_BLAKE2B)
        .add_data(script_hash)
        .add_op(OP_EQUAL)
        .to_smallvec();
    ScriptPublicKey::new(0, script)
}

/// Converts an Address to its matching standard ScriptPublicKey
pub fn pay_to_address_script(address: &Address) -> ScriptPublicKey {
    match address.version {
        Version::PubKey => {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&address.payload);
            pay_to_pubkey_script(&pk)
        }
        Version::PubKeyECDSA => {
            let mut pk = [0u8; 33];
            pk.copy_from_slice(&address.payload);
            pay_to_pubkey_ecdsa_script(&pk)
        }
        Version::ScriptHash => {
            let mut sh = [0u8; 32];
            sh.copy_from_slice(&address.payload);
            pay_to_script_hash_script(&sh)
        }
    }
}

/// Extracts an Address from a standard ScriptPublicKey (if matching a known template)
pub fn extract_script_pub_key_address(spk: &ScriptPublicKey, prefix: Prefix) -> Option<Address> {
    if spk.version != 0 {
        return None;
    }
    let s = spk.script.as_slice();

    // Standard P2PK: [32, <32 bytes>, OP_CHECKSIG] -> len = 34
    if s.len() == 34 && s[0] == 32 && s[33] == OP_CHECKSIG {
        return Address::new(prefix, Version::PubKey, &s[1..33]).ok();
    }

    // Standard P2PK ECDSA: [33, <33 bytes>, OP_CHECKSIG_ECDSA] -> len = 35
    if s.len() == 35 && s[0] == 33 && s[34] == OP_CHECKSIG_ECDSA {
        return Address::new(prefix, Version::PubKeyECDSA, &s[1..34]).ok();
    }

    // Standard P2SH: [OP_BLAKE2B, 32, <32 bytes>, OP_EQUAL] -> len = 35
    if s.len() == 35 && s[0] == OP_BLAKE2B && s[1] == 32 && s[34] == OP_EQUAL {
        return Address::new(prefix, Version::ScriptHash, &s[2..34]).ok();
    }

    None
}
