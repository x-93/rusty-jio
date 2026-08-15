use crate::result::CliResult;
use jio_wallet_core::Account;

pub async fn execute(_account: Option<&Account>) -> CliResult<()> {
    println!("Account command executed");
    Ok(())
}
