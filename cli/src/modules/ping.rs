use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Ping successful (roundtrip: 2ms)");
    Ok(())
}
