use crate::hex::FromHex;
use serde::de::{Deserializer, Visitor};
use std::fmt;

struct ByteVecVisitor;

impl<'de> Visitor<'de> for ByteVecVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array or hex string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Vec::<u8>::from_hex(v).map_err(serde::de::Error::custom)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_vec())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element()? {
            vec.push(elem);
        }
        Ok(vec)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        deserializer.deserialize_str(ByteVecVisitor)
    } else {
        deserializer.deserialize_byte_buf(ByteVecVisitor)
    }
}
