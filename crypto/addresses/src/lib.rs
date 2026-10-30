pub mod address;
pub mod prefix;

pub use address::{Address, AddressError, Version};
pub use prefix::Prefix;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_encode_and_decode_roundtrip() {
        let pubkey_bytes = vec![0x42; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, pubkey_bytes.clone());

        let encoded = addr.encode().expect("encoding should succeed");
        assert!(encoded.starts_with("jio:"));

        let decoded = Address::decode(&encoded).expect("decoding should succeed");
        assert_eq!(decoded.prefix, Prefix::Mainnet);
        assert_eq!(decoded.version, Version::PubKey);
        assert_eq!(decoded.payload, pubkey_bytes);
    }

    #[test]
    fn test_all_network_prefixes() {
        let key = vec![0x77; 32];
        for (prefix, expected_prefix_str) in [
            (Prefix::Mainnet, "jio:"),
            (Prefix::Testnet, "jiotest:"),
            (Prefix::Simnet, "jiosim:"),
            (Prefix::Devnet, "jiodev:"),
        ] {
            let addr = Address::new(prefix, Version::PubKey, key.clone());
            let encoded = addr.encode().unwrap();
            assert!(encoded.starts_with(expected_prefix_str));

            let decoded = Address::decode(&encoded).unwrap();
            assert_eq!(decoded.prefix, prefix);
            assert_eq!(decoded.version, Version::PubKey);
            assert_eq!(decoded.payload, key);
        }
    }

    #[test]
    fn test_ecdsa_and_scripthash_versions() {
        let ecdsa_key = vec![0x02; 33]; // 33-byte compressed secp256k1 key
        let addr_ecdsa = Address::new(Prefix::Mainnet, Version::PubKeyECDSA, ecdsa_key.clone());
        let encoded_ecdsa = addr_ecdsa.encode().unwrap();
        let decoded_ecdsa = Address::decode(&encoded_ecdsa).unwrap();
        assert_eq!(decoded_ecdsa.version, Version::PubKeyECDSA);
        assert_eq!(decoded_ecdsa.payload, ecdsa_key);

        let script_hash = vec![0xAA; 32]; // 32-byte P2SH script hash
        let addr_p2sh = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash.clone());
        let encoded_p2sh = addr_p2sh.encode().unwrap();
        let decoded_p2sh = Address::decode(&encoded_p2sh).unwrap();
        assert_eq!(decoded_p2sh.version, Version::ScriptHash);
        assert_eq!(decoded_p2sh.payload, script_hash);
    }

    #[test]
    fn test_invalid_address_rejection() {
        // Missing colon delimiter
        assert!(Address::decode("jiotestqp3d").is_err());

        // Unknown prefix
        assert!(Address::decode("bitcoin:qp3d").is_err());

        // Tampered checksum (modified last char)
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, vec![0x11; 32]);
        let mut encoded = addr.encode().unwrap();
        let last_char = encoded.pop().unwrap();
        let tampered_char = if last_char == 'q' { 'p' } else { 'q' };
        encoded.push(tampered_char);
        assert!(Address::decode(&encoded).is_err());
    }

    #[test]
    fn test_address_serde_json() {
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, vec![0x55; 32]);
        let json = serde_json::to_string(&addr).unwrap();
        let de: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, de);
    }
}
