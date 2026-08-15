use crate::error::CliError;
use crate::modules;
use crate::result::CliResult;
use jio_wallet_core::Wallet;

pub struct JioCli {
    pub active_wallet: Option<Wallet>,
}

impl JioCli {
    pub fn new() -> Self {
        Self {
            active_wallet: None,
        }
    }

    pub async fn handle_command(&mut self, cmd: &str, args: &[&str]) -> CliResult<()> {
        match cmd {
            "version" => modules::version::execute().await?,
            "ping" => modules::ping::execute().await?,
            "info" => modules::info::execute().await?,
            "help" => modules::help::execute().await?,
            "guide" => modules::guide::execute().await?,
            "settings" => modules::settings::execute().await?,
            "node" => modules::node::execute().await?,
            "create" => {
                let name = args.first().unwrap_or(&"default");
                let wallet = modules::create::execute(name).await?;
                self.active_wallet = Some(wallet);
            }
            "import" => {
                if let Some(phrase) = args.first() {
                    let wallet = modules::import::execute(phrase).await?;
                    self.active_wallet = Some(wallet);
                } else {
                    return Err(CliError::InvalidArgument("missing mnemonic phrase".to_string()));
                }
            }
            "export" => modules::export::execute(self.active_wallet.as_ref()).await?,
            "wallet" => modules::wallet::execute(self.active_wallet.as_ref()).await?,
            "address" => {
                let account = self.active_wallet.as_ref().and_then(|w| w.default_account());
                modules::address::execute(account).await?;
            }
            "new" => {
                let account = self.active_wallet.as_ref().and_then(|w| w.default_account());
                modules::new::execute(account).await?;
            }
            "balance" | "show" => {
                let account = self.active_wallet.as_ref().and_then(|w| w.default_account());
                modules::show::execute(account).await?;
            }
            "send" => {
                if args.len() >= 2 {
                    let to = args[0];
                    let amt: u64 = args[1].parse().map_err(|_| CliError::InvalidArgument("invalid amount".to_string()))?;
                    modules::send::execute(to, amt).await?;
                } else {
                    return Err(CliError::InvalidArgument("usage: send <address> <amount>".to_string()));
                }
            }
            "exit" | "quit" => modules::exit::execute().await?,
            _ => println!("Unknown command: '{}'. Type 'help' for a list of commands.", cmd),
        }
        Ok(())
    }
}
