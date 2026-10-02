use thiserror::Error;

pub const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddressError {
    #[error("Invalid character '{0}' in address")]
    InvalidCharacter(char),
    #[error("Invalid checksum in address")]
    InvalidChecksum,
    #[error("Missing separator ':' in address")]
    MissingSeparator,
    #[error("Invalid address prefix: {0}")]
    InvalidPrefix(String),
    #[error("Invalid address version: {0}")]
    InvalidVersion(u8),
    #[error("Invalid address payload length: {0}")]
    InvalidPayloadLength(usize),
}

/// Computes the polymod checksum over the prefix and 5-bit data payload.
pub fn polymod(prefix: &str, values: &[u8]) -> u64 {
    let mut c: u64 = 1;
    for b in prefix.bytes() {
        let c0 = (c >> 35) as u8;
        c = ((c & 0x07ffffffff) << 5) ^ ((b as u64) & 0x1f);
        c ^= polymod_step(c0);
    }
    // Separator 0
    let c0 = (c >> 35) as u8;
    c = (c & 0x07ffffffff) << 5;
    c ^= polymod_step(c0);

    for &v in values {
        let c0 = (c >> 35) as u8;
        c = ((c & 0x07ffffffff) << 5) ^ (v as u64);
        c ^= polymod_step(c0);
    }
    c
}

#[inline(always)]
fn polymod_step(c0: u8) -> u64 {
    let mut x: u64 = 0;
    if (c0 & 1) != 0 {
        x ^= 0x98f2bc8e61;
    }
    if (c0 & 2) != 0 {
        x ^= 0x79b76d99e2;
    }
    if (c0 & 4) != 0 {
        x ^= 0xf33e5fb3c4;
    }
    if (c0 & 8) != 0 {
        x ^= 0xae2eabe2a8;
    }
    if (c0 & 16) != 0 {
        x ^= 0x1e4f43e470;
    }
    x
}

/// Converts 8-bit bytes to 5-bit words or vice versa.
pub fn convert_bits(data: &[u8], frombits: u32, tobits: u32, pad: bool) -> Result<Vec<u8>, AddressError> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret = Vec::new();
    let maxv: u32 = (1 << tobits) - 1;

    for &value in data {
        acc = (acc << frombits) | (value as u32);
        bits += frombits;
        while bits >= tobits {
            bits -= tobits;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad {
        if bits > 0 {
            ret.push(((acc << (tobits - bits)) & maxv) as u8);
        }
    } else if bits >= frombits || ((acc << (tobits - bits)) & maxv) != 0 {
        return Err(AddressError::InvalidPayloadLength(data.len()));
    }

    Ok(ret)
}

/// Encodes prefix, version, and payload into Bech32 address string.
pub fn encode(prefix: &str, version: u8, payload: &[u8]) -> Result<String, AddressError> {
    let mut data5 = Vec::with_capacity(1 + payload.len() * 8 / 5 + 1);
    data5.push(version);
    let converted = convert_bits(payload, 8, 5, true)?;
    data5.extend_from_slice(&converted);

    // Compute 40-bit checksum (8 characters)
    let mut values_with_zeroes = data5.clone();
    values_with_zeroes.extend_from_slice(&[0u8; 8]);
    let mod_val = polymod(prefix, &values_with_zeroes) ^ 1;

    let mut checksum = [0u8; 8];
    for i in 0..8 {
        checksum[i] = ((mod_val >> (5 * (7 - i))) & 0x1f) as u8;
    }

    let mut result = String::with_capacity(prefix.len() + 1 + data5.len() + 8);
    result.push_str(prefix);
    result.push(':');
    for v in data5 {
        result.push(CHARSET[v as usize] as char);
    }
    for v in checksum {
        result.push(CHARSET[v as usize] as char);
    }
    Ok(result)
}

/// Decodes Bech32 address string into prefix, version, and payload.
pub fn decode(s: &str) -> Result<(String, u8, Vec<u8>), AddressError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(AddressError::MissingSeparator);
    }
    let prefix = parts[0].to_ascii_lowercase();
    let data_str = parts[1];
    if data_str.len() < 8 {
        return Err(AddressError::InvalidPayloadLength(data_str.len()));
    }

    let mut data5 = Vec::with_capacity(data_str.len());
    for c in data_str.chars() {
        let b = c.to_ascii_lowercase() as u8;
        if let Some(pos) = CHARSET.iter().position(|&x| x == b) {
            data5.push(pos as u8);
        } else {
            return Err(AddressError::InvalidCharacter(c));
        }
    }

    // Verify polymod checksum
    if polymod(&prefix, &data5) != 1 {
        return Err(AddressError::InvalidChecksum);
    }

    let payload_len = data5.len() - 8;
    let version = data5[0];
    let payload = convert_bits(&data5[1..payload_len], 5, 8, false)?;

    Ok((prefix, version, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bech32_encode_decode() {
        let prefix = "jio";
        let version = 0x00;
        let payload = [0x42u8; 32];
        let addr = encode(prefix, version, &payload).unwrap();
        assert!(addr.starts_with("jio:"));

        let (dec_prefix, dec_version, dec_payload) = decode(&addr).unwrap();
        assert_eq!(dec_prefix, prefix);
        assert_eq!(dec_version, version);
        assert_eq!(dec_payload.as_slice(), &payload);
    }
}
