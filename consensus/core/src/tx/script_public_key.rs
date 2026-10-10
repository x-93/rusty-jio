use borsh::{BorshDeserialize, BorshSerialize};
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};

pub type ScriptVec = SmallVec<[u8; 36]>;

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPublicKey {
    version: u16,
    script: ScriptVec,
}

impl BorshSerialize for ScriptPublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.version, writer)?;
        BorshSerialize::serialize(&(self.script.len() as u32), writer)?;
        writer.write_all(&self.script)
    }
}

impl BorshDeserialize for ScriptPublicKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let version = u16::deserialize_reader(reader)?;
        let len = u32::deserialize_reader(reader)? as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_vec(version, buf))
    }
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

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn script(&self) -> &[u8] {
        &self.script
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.script
    }
}

impl Debug for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptPublicKey {{ version: {}, script: ", self.version)?;
        for b in &self.script {
            write!(f, "{:02x}", b)?;
        }
        write!(f, " }}")
    }
}

impl Display for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.version)?;
        for b in &self.script {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl MemSizeEstimator for ScriptPublicKey {
    fn estimate_mem_bytes(&self) -> usize {
        let extra = if self.script.spilled() {
            self.script.capacity()
        } else {
            0
        };
        size_of::<Self>() + extra
    }
}
