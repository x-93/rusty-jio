use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Available commands: create, address, send, balance, connect, disconnect, info, ping, version, exit");
    Ok(())
}
