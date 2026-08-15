use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Disconnected from node.");
    Ok(())
}
