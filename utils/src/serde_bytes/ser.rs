use crate::hex::ToHex;
use serde::Serializer;

pub fn serialize<T, S>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    if serializer.is_human_readable() {
        serializer.serialize_str(&bytes.as_ref().to_hex())
    } else {
        serializer.serialize_bytes(bytes.as_ref())
    }
}
