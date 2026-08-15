use thiserror::Error;

const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

const GENERATOR: [u64; 5] = [
    0x98f2bc8e61,
    0x79b76d99e2,
    0xf33e5fb3c4,
    0xae2eabe2a8,
    0x1e4f43e470,
];

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum Bech32Error {
    #[error("invalid character '{0}' in bech32 string")]
    InvalidChar(char),
    #[error("invalid bech32 length: {0}")]
    InvalidLength(usize),
    #[error("missing separator ':' or '1'")]
    MissingSeparator,
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("invalid padding in 5-bit to 8-bit conversion")]
    InvalidPadding,
    #[error("invalid prefix: expected '{expected}', found '{actual}'")]
    InvalidPrefix { expected: String, actual: String },
}

fn polymod(values: &[u8]) -> u64 {
    let mut chk: u64 = 1;
    for &v in values {
        let b = chk >> 35;
        chk = ((chk & 0x07ffffffff) << 5) ^ (v as u64);
        for i in 0..5 {
            if ((b >> i) & 1) != 0 {
                chk ^= GENERATOR[i];
            }
        }
    }
    chk
}

fn hrp_expand(hrp: &str) -> Vec<u8> {
    let mut ret = Vec::with_capacity(hrp.len() + 1);
    for c in hrp.bytes() {
        ret.push(c & 0x1f);
    }
    ret.push(0);
    ret
}

pub fn convert_bits(data: &[u8], from_bits: u32, to_bits: u32, pad: bool) -> Result<Vec<u8>, Bech32Error> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret = Vec::new();
    let maxv: u32 = (1 << to_bits) - 1;

    for &value in data {
        let v = value as u32;
        if (v >> from_bits) != 0 {
            return Err(Bech32Error::InvalidPadding);
        }
        acc = (acc << from_bits) | v;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad {
        if bits > 0 {
            ret.push(((acc << (to_bits - bits)) & maxv) as u8);
        }
    } else if bits >= from_bits || ((acc << (to_bits - bits)) & maxv) != 0 {
        return Err(Bech32Error::InvalidPadding);
    }

    Ok(ret)
}

pub fn encode(hrp: &str, payload_5bit: &[u8]) -> String {
    let mut values = hrp_expand(hrp);
    values.extend_from_slice(payload_5bit);
    values.extend_from_slice(&[0u8; 8]);

    let chk = polymod(&values) ^ 1;
    let mut checksum = [0u8; 8];
    for i in 0..8 {
        checksum[i] = ((chk >> (5 * (7 - i))) & 0x1f) as u8;
    }

    let mut result = String::with_capacity(hrp.len() + 1 + payload_5bit.len() + 8);
    result.push_str(hrp);
    result.push(':');

    for &b in payload_5bit.iter().chain(checksum.iter()) {
        result.push(CHARSET[b as usize] as char);
    }

    result
}

pub fn decode(s: &str) -> Result<(String, Vec<u8>), Bech32Error> {
    let sep_pos = s.rfind(':').or_else(|| s.rfind('1')).ok_or(Bech32Error::MissingSeparator)?;
    let hrp = &s[..sep_pos];
    let data_str = &s[sep_pos + 1..];

    if data_str.len() < 8 {
        return Err(Bech32Error::InvalidLength(data_str.len()));
    }

    let mut values_5bit = Vec::with_capacity(data_str.len());
    for c in data_str.chars() {
        let lower = c.to_ascii_lowercase();
        let idx = CHARSET.iter().position(|&x| x == lower as u8).ok_or(Bech32Error::InvalidChar(c))?;
        values_5bit.push(idx as u8);
    }

    let mut chk_values = hrp_expand(hrp);
    chk_values.extend_from_slice(&values_5bit);

    if polymod(&chk_values) != 1 {
        return Err(Bech32Error::InvalidChecksum);
    }

    let payload_len = values_5bit.len() - 8;
    Ok((hrp.to_string(), values_5bit[..payload_len].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bech32_roundtrip() {
        let hrp = "jio";
        let raw = [42u8; 32];
        let converted_5bit = convert_bits(&raw, 8, 5, true).unwrap();
        let encoded = encode(hrp, &converted_5bit);
        assert!(encoded.starts_with("jio:"));

        let (decoded_hrp, decoded_5bit) = decode(&encoded).unwrap();
        assert_eq!(decoded_hrp, hrp);
        let decoded_8bit = convert_bits(&decoded_5bit, 5, 8, false).unwrap();
        assert_eq!(raw.to_vec(), decoded_8bit);
    }
}
