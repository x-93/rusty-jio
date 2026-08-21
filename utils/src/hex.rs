pub use faster_hex::Error;

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait FromHex: Sized {
    type Error;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error>;
}
