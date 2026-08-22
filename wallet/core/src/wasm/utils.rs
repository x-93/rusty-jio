use crate::result::Result;
use js_sys::BigInt;
use jio_consensus_core::network::{NetworkType, NetworkTypeT};
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bigint | number | HexString")]
    #[derive(Clone, Debug)]
    pub type ISompiToJio;
}

/// Convert a Jio string to Sompi represented by bigint.
/// This function provides correct precision handling and
/// can be used to parse user input.
/// @category Wallet SDK
#[wasm_bindgen(js_name = "jioToSompi")]
pub fn jio_to_sompi(jio: String) -> Option<BigInt> {
    crate::utils::try_jio_str_to_sompi(jio).ok().flatten().map(Into::into)
}

///
/// Convert Sompi to a string representation of the amount in Jio.
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToJioString")]
pub fn sompi_to_jio_string(sompi: ISompiToJio) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    Ok(crate::utils::sompi_to_jio_string(sompi))
}

///
/// Format a Sompi amount to a string representation of the amount in Jio with a suffix
/// based on the network type (e.g. `KAS` for mainnet, `TKAS` for testnet,
/// `SKAS` for simnet, `DKAS` for devnet).
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToJioStringWithSuffix")]
pub fn sompi_to_jio_string_with_suffix(sompi: ISompiToJio, network: &NetworkTypeT) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    let network_type = NetworkType::try_from(network)?;
    Ok(crate::utils::sompi_to_jio_string_with_suffix(sompi, &network_type))
}
