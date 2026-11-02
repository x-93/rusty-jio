use jio_wallet_cli::jio_wallet_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen(start)]
pub async fn load_jio_wallet_cli() -> Result<()> {
    workflow_wasm::panic::init_console_panic_hook();
    let options = Options { ..Options::default() };
    jio_wallet_cli(options).await?;
    Ok(())
}
