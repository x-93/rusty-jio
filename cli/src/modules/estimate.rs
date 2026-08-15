use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    println!("Fee estimate: 0.00001000 JIO/mass");
    Ok(())
}
