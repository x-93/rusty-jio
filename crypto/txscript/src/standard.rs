//! Standard script recognition and verification.

pub mod multisig;

pub use multisig::*;
pub use super::script_class::ScriptClass;
use jio_addresses::{Address, Version};
use super::opcodes::macros::*;

/// Checks if an output script adheres to standard network policies.
pub fn is_standard_output(script: &[u8]) -> bool {
    !matches!(ScriptClass::from_script(script), ScriptClass::NonStandard)
}

/// Creates a standard output script paying to the given address.
pub fn pay_to_address_script(address: &Address) -> Vec<u8> {
    match address.version {
        Version::PubKey => {
            let mut script = Vec::with_capacity(34);
            script.push(0x20); // OP_DATA_32
            script.extend_from_slice(&address.payload);
            script.push(OP_CHECKSIG);
            script
        }
        Version::PubKeyECDSA => {
            let mut script = Vec::with_capacity(35);
            script.push(0x21); // OP_DATA_33
            script.extend_from_slice(&address.payload);
            script.push(OP_CHECKSIG);
            script
        }
        Version::ScriptHash => {
            let mut script = Vec::with_capacity(35);
            script.push(OP_HASH256);
            script.push(0x20); // OP_DATA_32
            script.extend_from_slice(&address.payload);
            script.push(OP_EQUAL);
            script
        }
    }
}

/// Extracts destination address from a script if standard.
pub fn extract_script_pub_key_address(script: &[u8], prefix: jio_addresses::Prefix) -> Option<Address> {
    let class = ScriptClass::from_script(script);
    class.extract_address(script, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_addresses::Prefix;

    #[test]
    fn test_pay_to_address_script_roundtrip() {
        let pubkey_addr = Address::from_public_key(Prefix::Jio, &[0x42; 32]);
        let script = pay_to_address_script(&pubkey_addr);
        assert!(is_standard_output(&script));
        assert_eq!(ScriptClass::from_script(&script), ScriptClass::PubKey);

        let extracted = extract_script_pub_key_address(&script, Prefix::Jio).unwrap();
        assert_eq!(pubkey_addr, extracted);

        let ecdsa_addr = Address::from_public_key_ecdsa(Prefix::JioTest, &[0x33; 33]);
        let ecdsa_script = pay_to_address_script(&ecdsa_addr);
        assert!(is_standard_output(&ecdsa_script));
        assert_eq!(ScriptClass::from_script(&ecdsa_script), ScriptClass::PubKeyECDSA);

        let extracted_ecdsa = extract_script_pub_key_address(&ecdsa_script, Prefix::JioTest).unwrap();
        assert_eq!(ecdsa_addr, extracted_ecdsa);

        let sh_addr = Address::from_script_hash(Prefix::JioDev, &[0x77; 32]);
        let sh_script = pay_to_address_script(&sh_addr);
        assert!(is_standard_output(&sh_script));
        assert_eq!(ScriptClass::from_script(&sh_script), ScriptClass::ScriptHash);

        let extracted_sh = extract_script_pub_key_address(&sh_script, Prefix::JioDev).unwrap();
        assert_eq!(sh_addr, extracted_sh);
    }
}
