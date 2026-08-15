use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum FromHexError {
    #[error("odd number of digits in hex string: length {0}")]
    OddLength(usize),
    #[error("invalid hex character in input")]
    InvalidHexCharacter,
    #[error("expected {expected} bytes, but got {actual}")]
    InvalidLength { expected: usize, actual: usize },
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait FromHex: Sized {
    type Error;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error>;
}

impl ToHex for [u8] {
    fn to_hex(&self) -> String {
        encode_hex(self)
    }
}

impl ToHex for &[u8] {
    fn to_hex(&self) -> String {
        encode_hex(self)
    }
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        encode_hex(self)
    }
}

impl FromHex for Vec<u8> {
    type Error = FromHexError;
    fn from_hex(hex_str: &str) -> Result<Self, Self::Error> {
        decode_hex(hex_str)
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut buffer = vec![0u8; bytes.len() * 2];
    faster_hex::hex_encode(bytes, &mut buffer).expect("buffer has enough capacity");
    unsafe { String::from_utf8_unchecked(buffer) }
}

pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, FromHexError> {
    let hex_bytes = hex_str.as_bytes();
    if hex_bytes.len() % 2 != 0 {
        return Err(FromHexError::OddLength(hex_bytes.len()));
    }
    let mut dest = vec![0u8; hex_bytes.len() / 2];
    faster_hex::hex_decode(hex_bytes, &mut dest).map_err(|e| match e {
        faster_hex::Error::InvalidChar => FromHexError::InvalidHexCharacter,
        faster_hex::Error::InvalidLength(len) => FromHexError::OddLength(len),
        faster_hex::Error::Overflow => FromHexError::InvalidLength {
            expected: dest.len(),
            actual: hex_bytes.len() / 2,
        },
    })?;
    Ok(dest)
}

pub fn decode_to_slice(hex_str: &str, dest: &mut [u8]) -> Result<(), FromHexError> {
    let hex_bytes = hex_str.as_bytes();
    if hex_bytes.len() % 2 != 0 {
        return Err(FromHexError::OddLength(hex_bytes.len()));
    }
    if hex_bytes.len() / 2 != dest.len() {
        return Err(FromHexError::InvalidLength {
            expected: dest.len(),
            actual: hex_bytes.len() / 2,
        });
    }
    faster_hex::hex_decode(hex_bytes, dest).map_err(|e| match e {
        faster_hex::Error::InvalidChar => FromHexError::InvalidHexCharacter,
        faster_hex::Error::InvalidLength(len) => FromHexError::OddLength(len),
        faster_hex::Error::Overflow => FromHexError::InvalidLength {
            expected: dest.len(),
            actual: hex_bytes.len() / 2,
        },
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode_decode() {
        let raw = b"rusty-jio blockchain";
        let hex_str = encode_hex(raw);
        let decoded = decode_hex(&hex_str).unwrap();
        assert_eq!(raw.to_vec(), decoded);

        let mut fixed = [0u8; 20];
        decode_to_slice(&hex_str, &mut fixed).unwrap();
        assert_eq!(&fixed[..], raw);
    }

    #[test]
    fn test_odd_length() {
        let err = decode_hex("abc").unwrap_err();
        assert!(matches!(err, FromHexError::OddLength(3)));
    }

    #[test]
    fn test_invalid_char() {
        let err = decode_hex("zz").unwrap_err();
        assert!(matches!(err, FromHexError::InvalidHexCharacter));
    }
}
