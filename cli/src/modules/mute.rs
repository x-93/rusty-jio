use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Notifications muted.");
    Ok(())
}
