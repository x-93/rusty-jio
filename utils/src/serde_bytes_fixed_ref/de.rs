use serde::Deserializer;

pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    crate::serde_bytes_fixed::de::deserialize(deserializer)
}
