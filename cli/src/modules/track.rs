use crate::result::CliResult;

pub async fn execute(tx_id: &str) -> CliResult<()> {
    println!("Tracking transaction {}", tx_id);
    Ok(())
}
