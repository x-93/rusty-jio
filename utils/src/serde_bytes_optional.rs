use serde::{Deserializer, Serializer};

pub fn serialize<S, T>(opt: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    match opt {
        Some(bytes) => {
            if serializer.is_human_readable() {
                let hex_str = faster_hex::hex_string(bytes.as_ref());
                serializer.serialize_some(&hex_str)
            } else {
                serializer.serialize_some(bytes.as_ref())
            }
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptBytesVisitor;

    impl<'de> serde::de::Visitor<'de> for OptBytesVisitor {
        type Value = Option<Vec<u8>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional byte array or hex string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes = crate::serde_bytes::deserialize(deserializer)?;
            Ok(Some(bytes))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(OptBytesVisitor)
}
