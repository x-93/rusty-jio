use jio_cli::JioCli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Jio CLI Interactive Terminal v0.1.0 ===");
    println!("Type 'help' for commands or 'exit' to quit.");

    let mut cli = JioCli::new();
    cli.handle_command("version", &[]).await?;
    Ok(())
}
