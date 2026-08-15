use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Settings: devnet, default fee = 1000 sompi");
    Ok(())
}
