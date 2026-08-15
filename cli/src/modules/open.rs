use crate::result::CliResult;

pub async fn execute(wallet_name: &str) -> CliResult<()> {
    println!("Opened wallet '{}'", wallet_name);
    Ok(())
}
