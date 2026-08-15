pub mod cli;
pub mod error;
pub mod extensions;
pub mod helpers;
pub mod imports;
pub mod matchers;
pub mod modules;
pub mod notifier;
pub mod result;
pub mod utils;

pub use cli::*;
pub use error::*;
pub use result::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_command_handling() {
        let mut cli = JioCli::new();
        cli.handle_command("version", &[]).await.expect("version command works");
        cli.handle_command("create", &["my_wallet"]).await.expect("wallet created");
        assert!(cli.active_wallet.is_some());
    }
}
