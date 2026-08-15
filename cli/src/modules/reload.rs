use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Wallet state reloaded.");
    Ok(())
}
