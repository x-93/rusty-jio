use crate::result::CliResult;
use jio_addresses::Prefix;
use jio_bip32::ExtendedPrivateKey;
use jio_wallet_core::{Account, Wallet};
use jio_wallet_keys::mnemonic::JioMnemonic;
use jio_wallet_keys::xprv::XPrv;
use std::collections::HashMap;

pub async fn execute(mnemonic_phrase: &str) -> CliResult<Wallet> {
    let mnemonic = JioMnemonic::from_phrase(mnemonic_phrase)
        .map_err(|e| crate::error::CliError::Wallet(e))?;
    let seed = mnemonic.to_seed("");
    let master = ExtendedPrivateKey::new_master(&seed)
        .map_err(|e| crate::error::CliError::Wallet(e))?;
    let xprv = XPrv::new(master);

    let mut accounts = HashMap::new();
    let default_account = Account::new("default".to_string(), 0, xprv, Prefix::Devnet);
    accounts.insert(0, default_account);

    println!("Wallet successfully imported!");
    Ok(Wallet {
        name: "imported".to_string(),
        prefix: Prefix::Devnet,
        accounts,
        mnemonic,
    })
}
