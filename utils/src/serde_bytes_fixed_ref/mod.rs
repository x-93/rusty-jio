#[macro_export]
macro_rules! serde_impl_ser_fixed_bytes_ref {
    ($type:ident, $len:expr) => {
        #[cfg(feature = "serde")]
        impl serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&$crate::hex::ToHex::to_hex(self))
                } else {
                    serializer.serialize_bytes(self.as_ref())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! serde_impl_deser_fixed_bytes_ref {
    ($type:ident, $len:expr) => {
        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = String::deserialize(deserializer)?;
                    $crate::hex::FromHex::from_hex(&s).map_err(serde::de::Error::custom)
                } else {
                    struct BytesVisitor;
                    impl<'de> serde::de::Visitor<'de> for BytesVisitor {
                        type Value = $type;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(concat!("a ", stringify!($len), "-byte array"))
                        }

                        fn visit_bytes<E>(self, v: &[u8]) -> Result<$type, E>
                        where
                            E: serde::de::Error,
                        {
                            $type::try_from(v).map_err(serde::de::Error::custom)
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<$type, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            let mut bytes = [0u8; $len];
                            for (i, byte) in bytes.iter_mut().enumerate() {
                                *byte = seq
                                    .next_element()?
                                    .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                            }
                            Ok($type::from(bytes))
                        }
                    }
                    deserializer.deserialize_bytes(BytesVisitor)
                }
            }
        }
    };
}
