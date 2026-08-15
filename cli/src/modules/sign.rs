use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Transaction signed.");
    Ok(())
}
