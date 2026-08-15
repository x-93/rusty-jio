use crate::account::Account;
use jio_addresses::Prefix;
use jio_bip32::ExtendedPrivateKey;
use jio_wallet_keys::mnemonic::JioMnemonic;
use jio_wallet_keys::xprv::XPrv;
use std::collections::HashMap;

pub struct Wallet {
    pub name: String,
    pub prefix: Prefix,
    pub accounts: HashMap<u32, Account>,
    pub mnemonic: JioMnemonic,
}

impl Wallet {
    pub fn create_random(name: String, prefix: Prefix, password: &str) -> Result<Self, String> {
        let mnemonic = JioMnemonic::random(12)?;
        let seed = mnemonic.to_seed(password);
        let master = ExtendedPrivateKey::new_master(&seed)?;
        let xprv = XPrv::new(master);

        let mut accounts = HashMap::new();
        let default_account = Account::new("default".to_string(), 0, xprv, prefix.clone());
        accounts.insert(0, default_account);

        Ok(Self {
            name,
            prefix,
            accounts,
            mnemonic,
        })
    }

    pub fn default_account(&self) -> Option<&Account> {
        self.accounts.get(&0)
    }
}
