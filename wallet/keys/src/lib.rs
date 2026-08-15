pub mod bip39;
pub mod derivation;
pub mod keypair;
pub mod mnemonic;
pub mod xprv;
pub mod xpub;

pub use derivation::*;
pub use keypair::*;
pub use mnemonic::*;
pub use xprv::*;
pub use xpub::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_addresses::Prefix;

    #[test]
    fn test_mnemonic_and_key_derivation() {
        let mnemonic = JioMnemonic::random(12).expect("random mnemonic");
        let seed = mnemonic.to_seed("");
        let master = jio_bip32::ExtendedPrivateKey::new_master(&seed).expect("valid master");
        let xprv = XPrv::new(master);

        let path = DerivationPath::new(0, false, 0);
        let keypair = xprv.derive_path(&path).expect("derived keypair");

        let address = keypair.to_address(Prefix::Devnet);
        assert_eq!(address.prefix, Prefix::Devnet);
    }
}
