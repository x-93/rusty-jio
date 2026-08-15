use crate::result::CliResult;
use jio_wallet_core::Wallet;

pub async fn execute(wallet: Option<&Wallet>) -> CliResult<()> {
    if let Some(w) = wallet {
        println!("Exporting mnemonic for wallet '{}':", w.name);
        println!("{}", w.mnemonic.phrase());
    } else {
        println!("No wallet active");
    }
    Ok(())
}
