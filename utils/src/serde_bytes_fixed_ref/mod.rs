pub mod de;
pub mod ser;

pub use de::deserialize;
pub use ser::serialize;

#[macro_export]
macro_rules! serde_impl_ser_fixed_bytes_ref {
    ($t:ty, $size:expr) => {
        impl serde::Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $crate::serde_bytes_fixed_ref::ser::serialize(&&self.as_bytes(), serializer)
            }
        }
    };
}

#[macro_export]
macro_rules! serde_impl_deser_fixed_bytes_ref {
    ($t:ty, $size:expr) => {
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::serde_bytes_fixed_ref::de::deserialize(deserializer).map(Self::from)
            }
        }
    };
}
