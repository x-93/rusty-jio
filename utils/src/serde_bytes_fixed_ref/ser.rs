use serde::Serializer;

pub fn serialize<const N: usize, S>(bytes: &&[u8; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    crate::serde_bytes_fixed::ser::serialize(*bytes, serializer)
}
