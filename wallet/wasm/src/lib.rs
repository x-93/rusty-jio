use jio_cli_lib::jio_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_jio_wallet_cli() -> Result<()> {
    let options = Options { ..Options::default() };
    jio_cli(options, None).await?;
    Ok(())
}
