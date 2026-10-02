//! ScriptPublicKey represents the locking script and script version on an output.

use core::fmt;
use smallvec::SmallVec;
use jio_utils::ToHex;

pub type ScriptVec = SmallVec<[u8; 36]>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: ScriptVec,
}

impl ScriptPublicKey {
    pub fn new(version: u16, script: ScriptVec) -> Self {
        Self { version, script }
    }

    pub fn from_vec(version: u16, script: Vec<u8>) -> Self {
        Self {
            version,
            script: SmallVec::from_vec(script),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.script.as_slice()
    }

    pub fn len(&self) -> usize {
        self.script.len()
    }

    pub fn is_empty(&self) -> bool {
        self.script.is_empty()
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for ScriptPublicKey {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.version.serialize(writer)?;
        let bytes: &[u8] = self.script.as_slice();
        bytes.serialize(writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for ScriptPublicKey {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let version = u16::deserialize_reader(reader)?;
        let bytes: Vec<u8> = Vec::deserialize_reader(reader)?;
        Ok(Self::from_vec(version, bytes))
    }
}

impl fmt::Display for ScriptPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04x}:{}", self.version, self.script.to_hex())
    }
}

impl fmt::Debug for ScriptPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScriptPublicKey({})", self)
    }
}
