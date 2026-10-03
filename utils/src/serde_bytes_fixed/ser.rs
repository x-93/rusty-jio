use serde::ser::SerializeTuple;
use serde::Serializer;

pub fn serialize<S, const N: usize>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(N)?;
    for byte in bytes {
        tuple.serialize_element(byte)?;
    }
    tuple.end()
}
