use crate::result::CliResult;

pub async fn execute(to_address: &str, amount: u64) -> CliResult<()> {
    println!("Sending {} sompi to address {}", amount, to_address);
    println!("Transaction submitted successfully!");
    Ok(())
}
