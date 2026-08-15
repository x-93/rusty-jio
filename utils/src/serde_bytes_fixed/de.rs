use serde::de::{Error, Visitor};
use serde::Deserializer;
use std::fmt;

pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    struct FixedBytesVisitor<const N: usize>;

    impl<'de, const N: usize> Visitor<'de> for FixedBytesVisitor<N> {
        type Value = [u8; N];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a byte array of length {N} or hex string of length {}", N * 2)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let mut array = [0u8; N];
            crate::hex::decode_to_slice(v, &mut array).map_err(Error::custom)?;
            Ok(array)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v.len() != N {
                return Err(Error::invalid_length(v.len(), &self));
            }
            let mut array = [0u8; N];
            array.copy_from_slice(v);
            Ok(array)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut array = [0u8; N];
            for (i, item) in array.iter_mut().enumerate() {
                *item = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(i, &self))?;
            }
            Ok(array)
        }
    }

    if deserializer.is_human_readable() {
        deserializer.deserialize_str(FixedBytesVisitor::<N>)
    } else {
        deserializer.deserialize_bytes(FixedBytesVisitor::<N>)
    }
}
