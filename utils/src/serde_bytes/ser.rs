use serde::Serializer;

pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    if serializer.is_human_readable() {
        let hex_str = faster_hex::hex_string(bytes.as_ref());
        serializer.serialize_str(&hex_str)
    } else {
        serializer.serialize_bytes(bytes.as_ref())
    }
}
