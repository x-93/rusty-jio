use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Server address: 127.0.0.1:16110");
    Ok(())
}
