use crate::address::WalletAddress;
use crate::balance::AccountBalance;
use crate::keychain::KeyChain;
use jio_addresses::Prefix;
use jio_wallet_keys::xprv::XPrv;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Account {
    pub name: String,
    pub index: u32,
    pub keychain: KeyChain,
    pub balance: Arc<RwLock<AccountBalance>>,
}

impl Account {
    pub fn new(name: String, index: u32, xprv: XPrv, prefix: Prefix) -> Self {
        let keychain = KeyChain::new(xprv, index, prefix);
        Self {
            name,
            index,
            keychain,
            balance: Arc::new(RwLock::new(AccountBalance::default())),
        }
    }

    pub fn receive_address(&self) -> Result<WalletAddress, String> {
        self.keychain.new_receive_address()
    }

    pub fn change_address(&self) -> Result<WalletAddress, String> {
        self.keychain.new_change_address()
    }

    pub fn balance(&self) -> AccountBalance {
        self.balance.read().clone()
    }
}
