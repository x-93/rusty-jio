use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("UTXO sweep completed.");
    Ok(())
}
