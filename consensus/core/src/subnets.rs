use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

pub const SUBNETWORK_ID_SIZE: usize = 20;

/// A 20-byte identifier for consensus subnetworks (native, coinbase, registry, and application payloads).
#[derive(
    Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, BorshSerialize, BorshDeserialize,
)]
pub struct SubnetworkId(pub [u8; SUBNETWORK_ID_SIZE]);

pub const SUBNETWORK_ID_NATIVE: SubnetworkId = SubnetworkId([0; SUBNETWORK_ID_SIZE]);
pub const SUBNETWORK_ID_COINBASE: SubnetworkId =
    SubnetworkId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
pub const SUBNETWORK_ID_REGISTRY: SubnetworkId =
    SubnetworkId([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

impl SubnetworkId {
    pub const fn from_bytes(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }

    pub const fn from_byte(b: u8) -> Self {
        let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
        bytes[0] = b;
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SUBNETWORK_ID_SIZE] {
        &self.0
    }

    pub fn is_native(&self) -> bool {
        self == &SUBNETWORK_ID_NATIVE
    }

    pub fn is_coinbase(&self) -> bool {
        self == &SUBNETWORK_ID_COINBASE
    }

    pub fn is_builtin(&self) -> bool {
        self.is_native() || self.is_coinbase() || self == &SUBNETWORK_ID_REGISTRY
    }

    pub fn is_builtin_or_native(&self) -> bool {
        self.is_builtin()
    }
}

impl AsRef<[u8]> for SubnetworkId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; SUBNETWORK_ID_SIZE]> for SubnetworkId {
    fn from(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }
}

impl Display for SubnetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut hex = [0u8; SUBNETWORK_ID_SIZE * 2];
        faster_hex::hex_encode(&self.0, &mut hex).expect("hex encode subnetwork id");
        f.write_str(unsafe { std::str::from_utf8_unchecked(&hex) })
    }
}

impl Debug for SubnetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SubnetworkId({})", self)
    }
}

impl FromStr for SubnetworkId {
    type Err = faster_hex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
        faster_hex::hex_decode(s.as_bytes(), &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl serde::Serialize for SubnetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for SubnetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String as serde::Deserialize>::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            struct BytesVisitor;
            impl<'de> serde::de::Visitor<'de> for BytesVisitor {
                type Value = SubnetworkId;

                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    formatter.write_str("a 20-byte array")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<SubnetworkId, E>
                where
                    E: serde::de::Error,
                {
                    if v.len() != SUBNETWORK_ID_SIZE {
                        return Err(serde::de::Error::invalid_length(v.len(), &self));
                    }
                    let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
                    bytes.copy_from_slice(v);
                    Ok(SubnetworkId(bytes))
                }
            }
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subnetwork_constants() {
        assert!(SUBNETWORK_ID_NATIVE.is_native());
        assert!(SUBNETWORK_ID_COINBASE.is_coinbase());
        assert!(SUBNETWORK_ID_NATIVE.is_builtin());
        assert!(SUBNETWORK_ID_COINBASE.is_builtin());
        assert!(SUBNETWORK_ID_REGISTRY.is_builtin());

        let custom = SubnetworkId::from_byte(0x42);
        assert!(!custom.is_builtin());
        assert!(!custom.is_native());
    }

    #[test]
    fn test_hex_conversions() {
        let subnetwork_id = SubnetworkId::from_byte(0x55);
        let s = subnetwork_id.to_string();
        let parsed = SubnetworkId::from_str(&s).unwrap();
        assert_eq!(subnetwork_id, parsed);
    }
}
