use jio_wallet_cli::{jio_wallet_cli, TerminalOptions};

#[tokio::main]
async fn main() {
    let result = jio_wallet_cli(TerminalOptions::new().with_prompt("$ ")).await;
    if let Err(err) = result {
        println!("{err}");
    }
}
