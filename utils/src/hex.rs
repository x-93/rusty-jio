use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    InvalidLength,
    InvalidChar(char),
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexError::InvalidLength => write!(f, "Invalid hex string length (must be even)"),
            HexError::InvalidChar(c) => write!(f, "Invalid hex character: '{}'", c),
        }
    }
}

impl std::error::Error for HexError {}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for [u8] {
    fn to_hex(&self) -> String {
        self.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        self.as_slice().to_hex()
    }
}

pub trait FromHex: Sized {
    type Error;
    fn from_hex(hex: &str) -> Result<Self, Self::Error>;
}

impl FromHex for Vec<u8> {
    type Error = HexError;

    fn from_hex(hex: &str) -> Result<Self, Self::Error> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if !hex.len().is_multiple_of(2) {
            return Err(HexError::InvalidLength);
        }
        let mut bytes = Vec::with_capacity(hex.len() / 2);
        for chunk in hex.as_bytes().chunks(2) {
            let high = parse_hex_digit(chunk[0] as char)?;
            let low = parse_hex_digit(chunk[1] as char)?;
            bytes.push((high << 4) | low);
        }
        Ok(bytes)
    }
}

fn parse_hex_digit(c: char) -> Result<u8, HexError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => Err(HexError::InvalidChar(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_and_from_hex() {
        let data = vec![0x00, 0x12, 0xab, 0xff];
        let hex_str = data.to_hex();
        assert_eq!(hex_str, "0012abff");

        let parsed = Vec::from_hex(&hex_str).unwrap();
        assert_eq!(parsed, data);

        let parsed_with_prefix = Vec::from_hex("0x0012abff").unwrap();
        assert_eq!(parsed_with_prefix, data);

        assert!(Vec::from_hex("123").is_err());
        assert!(Vec::from_hex("12zz").is_err());
    }
}
