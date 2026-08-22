use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

pub type ScriptVec = Vec<u8>;

/// Represents a script public key (locking script) on the Jio network.
#[derive(
    Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSerialize, BorshDeserialize,
)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: ScriptVec,
}

impl ScriptPublicKey {
    pub const fn new(version: u16, script: ScriptVec) -> Self {
        Self { version, script }
    }

    pub fn from_vec(version: u16, script: Vec<u8>) -> Self {
        Self { version, script }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
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
    pub fn is_op_true(&self) -> bool {
        self.version == 0 && self.script.len() == 1 && self.script[0] == 0x51
    }
}

impl Display for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut hex = vec![0u8; self.script.len() * 2];
        faster_hex::hex_encode(&self.script, &mut hex).expect("hex encode script public key");
        write!(f, "{:04x} {}", self.version, unsafe {
            std::str::from_utf8_unchecked(&hex)
        })
    }
}

impl Debug for ScriptPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Serialize for ScriptPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Helper {
                version: u16,
                script_public_key: String,
            }
            let mut hex = vec![0u8; self.script.len() * 2];
            faster_hex::hex_encode(&self.script, &mut hex).expect("hex encode script public key");
            let hex_str = unsafe { std::str::from_utf8_unchecked(&hex) };
            Helper {
                version: self.version,
                script_public_key: hex_str.to_string(),
            }
            .serialize(serializer)
        } else {
            #[derive(Serialize)]
            struct BinaryHelper<'a> {
                version: u16,
                script: &'a [u8],
            }
            BinaryHelper {
                version: self.version,
                script: &self.script,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ScriptPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Helper {
                version: u16,
                script_public_key: String,
            }
            let helper = Helper::deserialize(deserializer)?;
            let mut script = vec![0u8; helper.script_public_key.len() / 2];
            faster_hex::hex_decode(helper.script_public_key.as_bytes(), &mut script)
                .map_err(serde::de::Error::custom)?;
            Ok(Self {
                version: helper.version,
                script,
            })
        } else {
            #[derive(Deserialize)]
            struct BinaryHelper {
                version: u16,
                script: Vec<u8>,
            }
            let helper = BinaryHelper::deserialize(deserializer)?;
            Ok(Self {
                version: helper.version,
                script: helper.script,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_public_key_ser() {
        let spk = ScriptPublicKey::new(0, vec![0x51]);
        assert!(spk.is_op_true());
        let json = serde_json::to_string(&spk).unwrap();
        let deser: ScriptPublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(spk, deser);
    }
}
