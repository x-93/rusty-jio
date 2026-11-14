// consensus/core/src/tx/script_public_key.rs
//
// ScriptPublicKey represents the locking script of a UTXO in the BlockDAG.
// Optimized for zero heap allocations for standard P2PK/P2SH scripts using SmallVec.

use borsh::{BorshDeserialize, BorshSerialize};
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::mem::size_of;
use std::ops::Deref;

/// SmallVec holding up to 36 bytes inline without heap allocations.
/// Standard P2PK scripts are 34 bytes (32-byte pubkey + 1 push op + 1 OP_CHECKSIG).
pub type ScriptVec = SmallVec<[u8; 36]>;

pub type ScriptPublicKeyVersion = u16;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPublicKey {
    pub version: ScriptPublicKeyVersion,
    pub script: ScriptVec,
}

impl ScriptPublicKey {
    #[inline(always)]
    pub const fn new_by_val(version: ScriptPublicKeyVersion, script: ScriptVec) -> Self {
        Self { version, script }
    }

    #[inline(always)]
    pub fn new(version: ScriptPublicKeyVersion, script: ScriptVec) -> Self {
        Self { version, script }
    }

    #[inline(always)]
    pub fn from_vec(version: ScriptPublicKeyVersion, script: Vec<u8>) -> Self {
        Self {
            version,
            script: SmallVec::from_vec(script),
        }
    }

    #[inline(always)]
    pub fn version(&self) -> ScriptPublicKeyVersion {
        self.version
    }

    #[inline(always)]
    pub fn script(&self) -> &[u8] {
        &self.script
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.script.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.script.is_empty()
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.script
    }
}

impl Deref for ScriptPublicKey {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.script
    }
}

impl AsRef<[u8]> for ScriptPublicKey {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.script
    }
}

impl Display for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut hex = vec![0u8; self.script.len() * 2];
        faster_hex::hex_encode(&self.script, &mut hex).map_err(|_| std::fmt::Error)?;
        let hex_str = std::str::from_utf8(&hex).map_err(|_| std::fmt::Error)?;
        write!(f, "{}:{}", self.version, hex_str)
    }
}

impl std::fmt::Debug for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

// Custom canonical Borsh serialization for byte-level determinism
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

impl MemSizeEstimator for ScriptPublicKey {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + if self.script.spilled() {
                self.script.capacity()
            } else {
                0
            }
    }
}

// Seamless interoperability with jio-txscript::ScriptPublicKey
impl From<jio_txscript::ScriptPublicKey> for ScriptPublicKey {
    #[inline]
    fn from(spk: jio_txscript::ScriptPublicKey) -> Self {
        Self {
            version: spk.version,
            script: spk.script,
        }
    }
}

impl From<ScriptPublicKey> for jio_txscript::ScriptPublicKey {
    #[inline]
    fn from(spk: ScriptPublicKey) -> Self {
        Self {
            version: spk.version,
            script: spk.script,
        }
    }
}

impl From<(u16, Vec<u8>)> for ScriptPublicKey {
    #[inline]
    fn from((version, script): (u16, Vec<u8>)) -> Self {
        Self::from_vec(version, script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_public_key_creation() {
        let spk = ScriptPublicKey::from_vec(0, vec![0xaa, 0xbb, 0xcc]);
        assert_eq!(spk.version(), 0);
        assert_eq!(spk.len(), 3);
        assert!(!spk.is_empty());
        assert_eq!(spk.as_slice(), &[0xaa, 0xbb, 0xcc]);
        assert_eq!(spk.to_string(), "0:aabbcc");
    }

    #[test]
    fn test_script_public_key_borsh_roundtrip() {
        let original = ScriptPublicKey::from_vec(1, vec![0x01, 0x02, 0x03, 0x04]);
        let mut bytes = Vec::new();
        BorshSerialize::serialize(&original, &mut bytes).expect("serialization succeeds");

        let deserialized = <ScriptPublicKey as BorshDeserialize>::deserialize(&mut bytes.as_slice()).expect("deserialization succeeds");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_txscript_conversion() {
        let txscript_spk = jio_txscript::pay_to_pubkey_script(&[42u8; 32]);
        let core_spk: ScriptPublicKey = txscript_spk.clone().into();
        assert_eq!(core_spk.version, txscript_spk.version);
        assert_eq!(core_spk.script.as_slice(), txscript_spk.script.as_slice());

        let back_to_txscript: jio_txscript::ScriptPublicKey = core_spk.into();
        assert_eq!(back_to_txscript, txscript_spk);
    }
}
