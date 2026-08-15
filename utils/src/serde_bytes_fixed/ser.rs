use serde::Serializer;

pub fn serialize<const N: usize, S>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        let hex_str = faster_hex::hex_string(bytes);
        serializer.serialize_str(&hex_str)
    } else {
        serializer.serialize_bytes(bytes)
    }
}
