use crate::result::CliResult;
use jio_wallet_core::Account;

pub async fn execute(account: Option<&Account>) -> CliResult<()> {
    if let Some(acc) = account {
        let bal = acc.balance();
        println!("Account '{}': mature = {}, pending = {}", acc.name, bal.mature, bal.pending);
    }
    Ok(())
}
