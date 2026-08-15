use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Node Info: Jio Node v0.1.0 | Network: Devnet | Synced: true");
    Ok(())
}
