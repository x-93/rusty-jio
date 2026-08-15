use crate::result::CliResult;
use jio_addresses::Prefix;
use jio_wallet_core::Wallet;

pub async fn execute(wallet_name: &str) -> CliResult<Wallet> {
    println!("Creating new wallet '{}'...", wallet_name);
    let wallet = Wallet::create_random(wallet_name.to_string(), Prefix::Devnet, "")
        .map_err(|e| crate::error::CliError::Wallet(e))?;
    println!("Wallet created successfully!");
    println!("Mnemonic: {}", wallet.mnemonic.phrase());
    Ok(wallet)
}
