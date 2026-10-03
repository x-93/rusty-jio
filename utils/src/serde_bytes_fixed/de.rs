use serde::de::{Deserializer, Visitor};
use std::fmt;

struct FixedByteArrayVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for FixedByteArrayVisitor<N> {
    type Value = [u8; N];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a byte array of length {}", N)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == N {
            let mut arr = [0u8; N];
            arr.copy_from_slice(v);
            Ok(arr)
        } else {
            Err(serde::de::Error::invalid_length(v.len(), &self))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut arr = [0u8; N];
        for (i, item) in arr.iter_mut().enumerate() {
            *item = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
        }
        Ok(arr)
    }
}

pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_tuple(N, FixedByteArrayVisitor::<N>)
}
