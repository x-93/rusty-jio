use crate::address::WalletAddress;
use jio_addresses::Prefix;
use jio_wallet_keys::derivation::DerivationPath;
use jio_wallet_keys::keypair::KeyPair;
use jio_wallet_keys::xprv::XPrv;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct KeyChain {
    pub xprv: XPrv,
    pub account_index: u32,
    pub prefix: Prefix,
    pub next_receive_index: AtomicU32,
    pub next_change_index: AtomicU32,
}

impl KeyChain {
    pub fn new(xprv: XPrv, account_index: u32, prefix: Prefix) -> Self {
        Self {
            xprv,
            account_index,
            prefix,
            next_receive_index: AtomicU32::new(0),
            next_change_index: AtomicU32::new(0),
        }
    }

    pub fn new_receive_address(&self) -> Result<WalletAddress, String> {
        let index = self.next_receive_index.fetch_add(1, Ordering::SeqCst);
        let path = DerivationPath::new(self.account_index, false, index);
        let keypair = self.xprv.derive_path(&path)?;
        let address = keypair.to_address(self.prefix.clone());
        Ok(WalletAddress {
            address,
            index,
            is_change: false,
        })
    }

    pub fn new_change_address(&self) -> Result<WalletAddress, String> {
        let index = self.next_change_index.fetch_add(1, Ordering::SeqCst);
        let path = DerivationPath::new(self.account_index, true, index);
        let keypair = self.xprv.derive_path(&path)?;
        let address = keypair.to_address(self.prefix.clone());
        Ok(WalletAddress {
            address,
            index,
            is_change: true,
        })
    }

    pub fn get_keypair(&self, is_change: bool, index: u32) -> Result<KeyPair, String> {
        let path = DerivationPath::new(self.account_index, is_change, index);
        self.xprv.derive_path(&path)
    }
}
