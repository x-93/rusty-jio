use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Broadcast command executed");
    Ok(())
}
