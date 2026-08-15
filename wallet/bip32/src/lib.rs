pub mod child_number;
pub mod extended_key;
pub mod prefix;
pub mod secret_key;

pub use child_number::*;
pub use extended_key::*;
pub use prefix::*;
pub use secret_key::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip32_master_derivation() {
        let seed = [0x42u8; 64];
        let master = ExtendedPrivateKey::new_master(&seed).expect("valid master");
        assert_eq!(master.depth, 0);

        let child = master.derive_child(ChildNumber::hardened(44)).expect("valid child");
        assert_eq!(child.depth, 1);
        assert!(child.child_number.is_hardened());

        let pubkey = child.to_public_key();
        assert_eq!(pubkey.depth, 1);
    }
}
