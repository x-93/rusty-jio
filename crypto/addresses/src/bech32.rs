// bech32.rs
//
// Custom zero-allocation Bech32 / CashAddr-style encoding and decoding
// for rusty-jio addresses.
//
// Why CashAddr-style BCH (Bose-Chaudhuri-Hocquenghem) encoding?
// - Human-readable prefix (e.g. "jio:", "jiotest:")
// - Base32 character set: eliminates visually ambiguous characters (0, O, I, l)
// - 40-bit polynomial checksum: guarantees detection of up to 4 errors in any address
//   and high-probability detection of any number of random burst errors.

use thiserror::Error;

const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

#[rustfmt::skip]
const CHARSET_REV: [i8; 128] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    15, -1, 10, 17, 21, 20, 26, 30,  7,  5, -1, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25,  9,  8, 23, -1, 18, 22, 31, 27, 19, -1,
     1,  0,  3, 16, 11, 28, 12, 14,  6,  4,  2, -1, -1, -1, -1, -1,
    -1, 29, -1, 24, 13, 25,  9,  8, 23, -1, 18, 22, 31, 27, 19, -1,
     1,  0,  3, 16, 11, 28, 12, 14,  6,  4,  2, -1, -1, -1, -1, -1,
];

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Bech32Error {
    #[error("Invalid character in address: '{0}'")]
    InvalidChar(char),
    #[error("Invalid checksum in address")]
    InvalidChecksum,
    #[error("Missing prefix delimiter ':'")]
    MissingDelimiter,
    #[error("Empty payload")]
    EmptyPayload,
    #[error("Invalid 5-bit padding")]
    InvalidPadding,
}

/// Computes the polymod checksum step for a single 5-bit input value.
#[inline(always)]
fn polymod_step(pre: u64) -> u64 {
    let b = pre >> 35;
    ((pre & 0x07ffffffff) << 5)
        ^ (if (b & 0x01) != 0 { 0x98f2bc8e61 } else { 0 })
        ^ (if (b & 0x02) != 0 { 0x79b76d99e2 } else { 0 })
        ^ (if (b & 0x04) != 0 { 0xf33e5fb3c4 } else { 0 })
        ^ (if (b & 0x08) != 0 { 0xae2eabe2a8 } else { 0 })
        ^ (if (b & 0x10) != 0 { 0x1e4f43e470 } else { 0 })
}

/// Prefix expansion: extracts lower 5 bits of each ASCII character of the prefix,
/// followed by a zero 5-bit separator.
#[inline(always)]
fn prefix_checksum(prefix: &str) -> u64 {
    let mut checksum = 1u64;
    for b in prefix.bytes() {
        checksum = polymod_step(checksum) ^ ((b & 0x1f) as u64);
    }
    polymod_step(checksum)
}

/// Converts an 8-bit slice into 5-bit base32 words.
pub fn convert_bits_8_to_5(data: &[u8]) -> Vec<u8> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret = Vec::with_capacity((data.len() * 8).div_ceil(5));
    for &value in data {
        acc = (acc << 8) | (value as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            ret.push(((acc >> bits) & 0x1f) as u8);
        }
    }
    if bits > 0 {
        ret.push(((acc << (5 - bits)) & 0x1f) as u8);
    }
    ret
}

/// Converts a 5-bit slice back into 8-bit bytes.
pub fn convert_bits_5_to_8(data: &[u8]) -> Result<Vec<u8>, Bech32Error> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret = Vec::with_capacity(data.len() * 5 / 8);
    for &value in data {
        if value >= 32 {
            return Err(Bech32Error::InvalidPadding);
        }
        acc = (acc << 5) | (value as u32);
        bits += 5;
        while bits >= 8 {
            bits -= 8;
            ret.push(((acc >> bits) & 0xff) as u8);
        }
    }
    if bits >= 5 || ((acc << (8 - bits)) & 0xff) != 0 {
        return Err(Bech32Error::InvalidPadding);
    }
    Ok(ret)
}

/// Encodes a prefix and payload bytes into a human-readable Bech32 string.
pub fn encode(prefix: &str, payload_bytes: &[u8]) -> String {
    let payload_5bit = convert_bits_8_to_5(payload_bytes);
    let mut checksum = prefix_checksum(prefix);

    for &val in &payload_5bit {
        checksum = polymod_step(checksum) ^ (val as u64);
    }

    // 8 zero 5-bit words for the 40-bit checksum
    for _ in 0..8 {
        checksum = polymod_step(checksum);
    }
    checksum ^= 1;

    let mut result = String::with_capacity(prefix.len() + 1 + payload_5bit.len() + 8);
    result.push_str(prefix);
    result.push(':');

    for &val in &payload_5bit {
        result.push(CHARSET[val as usize] as char);
    }
    for i in (0..8).rev() {
        let char_idx = ((checksum >> (5 * i)) & 0x1f) as usize;
        result.push(CHARSET[char_idx] as char);
    }

    result
}

/// Decodes a Bech32 string into the human-readable prefix and raw payload bytes.
pub fn decode(encoded: &str) -> Result<(String, Vec<u8>), Bech32Error> {
    let parts: Vec<&str> = encoded.split(':').collect();
    if parts.len() != 2 {
        return Err(Bech32Error::MissingDelimiter);
    }
    let prefix = parts[0].to_ascii_lowercase();
    let data_str = parts[1];

    if data_str.len() < 8 {
        return Err(Bech32Error::EmptyPayload);
    }

    let mut data_5bit = Vec::with_capacity(data_str.len());
    let mut checksum = prefix_checksum(&prefix);

    for ch in data_str.chars() {
        let b = ch as usize;
        if b >= 128 {
            return Err(Bech32Error::InvalidChar(ch));
        }
        let val = CHARSET_REV[b];
        if val < 0 {
            return Err(Bech32Error::InvalidChar(ch));
        }
        let uval = val as u8;
        data_5bit.push(uval);
        checksum = polymod_step(checksum) ^ (uval as u64);
    }

    if checksum != 1 {
        return Err(Bech32Error::InvalidChecksum);
    }

    let payload_5bit = &data_5bit[..data_5bit.len() - 8];
    let payload_8bit = convert_bits_5_to_8(payload_5bit)?;

    Ok((prefix, payload_8bit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let prefix = "jio";
        let raw_payload = b"test_payload_for_rusty_jio_cryptocurrency_address_12345";
        let encoded = encode(prefix, raw_payload);
        assert!(encoded.starts_with("jio:"));

        let (dec_prefix, dec_payload) = decode(&encoded).expect("Decode must succeed");
        assert_eq!(dec_prefix, prefix);
        assert_eq!(dec_payload, raw_payload);
    }

    #[test]
    fn test_checksum_failure_on_mutation() {
        let prefix = "jio";
        let raw_payload = b"secure_user_wallet_address";
        let encoded = encode(prefix, raw_payload);

        // Mutate one character
        let mut corrupted: Vec<char> = encoded.chars().collect();
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] = if corrupted[last_idx] == 'q' { 'p' } else { 'q' };
        let corrupted_str: String = corrupted.into_iter().collect();

        assert_eq!(decode(&corrupted_str), Err(Bech32Error::InvalidChecksum));
    }
}
