use crate::result::CliResult;

pub async fn execute(node_url: &str) -> CliResult<()> {
    println!("Connecting to Jio node at {}", node_url);
    Ok(())
}
