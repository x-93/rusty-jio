//! Serde serialization for Option<Vec<u8>>.

use serde::{Deserializer, Serializer};

pub fn serialize<S>(opt: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(bytes) => serializer.serialize_some(&super::hex::ToHex::to_hex(bytes)),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_str: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    match opt_str {
        Some(s) => super::hex::FromHex::from_hex(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
