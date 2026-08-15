use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Node halt command sent.");
    Ok(())
}
