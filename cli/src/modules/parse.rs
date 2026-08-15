use crate::result::CliResult;

pub async fn execute(tx_hex: &str) -> CliResult<()> {
    println!("Parsing tx hex (len: {})", tx_hex.len());
    Ok(())
}
