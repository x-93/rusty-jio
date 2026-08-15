use crate::result::CliResult;

pub async fn execute(to_address: &str, amount: u64) -> CliResult<()> {
    println!("Transferring {} sompi to {}", amount, to_address);
    Ok(())
}
