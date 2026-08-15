use jio_utils::hex::{decode_to_slice, FromHex, FromHexError, ToHex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

pub const SUBNETWORK_ID_SIZE: usize = 20;

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubnetworkId(pub [u8; SUBNETWORK_ID_SIZE]);

pub const SUBNETWORK_ID_NATIVE: SubnetworkId = SubnetworkId([0u8; SUBNETWORK_ID_SIZE]);

pub const SUBNETWORK_ID_COINBASE: SubnetworkId = {
    let mut id = [0u8; SUBNETWORK_ID_SIZE];
    id[0] = 1;
    SubnetworkId(id)
};

pub const SUBNETWORK_ID_REGISTRY: SubnetworkId = {
    let mut id = [0u8; SUBNETWORK_ID_SIZE];
    id[0] = 2;
    SubnetworkId(id)
};

impl SubnetworkId {
    pub const fn from_byte(b: u8) -> Self {
        let mut id = [0u8; SUBNETWORK_ID_SIZE];
        id[0] = b;
        Self(id)
    }

    pub const fn from_bytes(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SUBNETWORK_ID_SIZE] {
        &self.0
    }

    pub fn is_native(&self) -> bool {
        *self == SUBNETWORK_ID_NATIVE
    }

    pub fn is_coinbase(&self) -> bool {
        *self == SUBNETWORK_ID_COINBASE
    }

    pub fn is_builtin(&self) -> bool {
        self.is_native() || self.is_coinbase() || *self == SUBNETWORK_ID_REGISTRY
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

impl FromHex for SubnetworkId {
    type Error = FromHexError;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
        decode_to_slice(hex_str, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl FromStr for SubnetworkId {
    type Err = FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl ToHex for SubnetworkId {
    fn to_hex(&self) -> String {
        faster_hex::hex_string(&self.0)
    }
}

impl fmt::Display for SubnetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for SubnetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubnetworkId({})", self.to_hex())
    }
}

impl Serialize for SubnetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for SubnetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <[u8; SUBNETWORK_ID_SIZE]>::deserialize(deserializer)?;
            Ok(Self(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subnetwork_id_properties() {
        assert!(SUBNETWORK_ID_NATIVE.is_native());
        assert!(!SUBNETWORK_ID_NATIVE.is_coinbase());
        assert!(SUBNETWORK_ID_COINBASE.is_coinbase());

        let hex = SUBNETWORK_ID_COINBASE.to_hex();
        let parsed = SubnetworkId::from_str(&hex).unwrap();
        assert_eq!(SUBNETWORK_ID_COINBASE, parsed);
    }
}
