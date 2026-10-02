use serde::Serializer;

pub fn serialize<T, S>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_bytes(bytes.as_ref())
}
