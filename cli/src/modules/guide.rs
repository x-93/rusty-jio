use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Jio CLI Interactive Guide");
    println!("1. 'create <wallet_name>' - Create a new wallet");
    println!("2. 'address'              - View receiving address");
    println!("3. 'send <addr> <amt>'    - Transfer funds");
    Ok(())
}
