use crate::result::CliResult;
use jio_wallet_core::Wallet;

pub async fn execute(wallet: Option<&Wallet>) -> CliResult<()> {
    if let Some(w) = wallet {
        println!("Current Active Wallet: {}", w.name);
    } else {
        println!("No wallet currently open.");
    }
    Ok(())
}
