use crate::result::CliResult;
use jio_wallet_core::Account;

pub async fn execute(account: Option<&Account>) -> CliResult<()> {
    if let Some(acc) = account {
        let addr = acc.receive_address().map_err(|e| crate::error::CliError::Wallet(e))?;
        println!("Receive Address: {}", addr.address);
    } else {
        println!("No wallet account active");
    }
    Ok(())
}
