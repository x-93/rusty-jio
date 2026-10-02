pub mod de;
pub mod ser;

pub use de::deserialize;
pub use ser::serialize;

#[macro_export]
macro_rules! serde_impl_ser_fixed_bytes_ref {
    ($t:ty, $size:expr) => {
        #[cfg(feature = "serde")]
        impl serde::Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&self.to_string())
                } else {
                    $crate::serde_bytes_fixed_ref::serialize(&self.as_ref(), serializer)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! serde_impl_deser_fixed_bytes_ref {
    ($t:ty, $size:expr) => {
        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = <&str>::deserialize(deserializer)?;
                    <$t as std::str::FromStr>::from_str(s).map_err(serde::de::Error::custom)
                } else {
                    let bytes = $crate::serde_bytes_fixed_ref::deserialize::<'de, D, $size>(deserializer)?;
                    Ok(Self::from(bytes))
                }
            }
        }
    };
}
