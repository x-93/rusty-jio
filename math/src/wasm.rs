#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::uint::Uint256;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Uint256 {
    #[wasm_bindgen(js_name = toBigInt)]
    pub fn to_bigint(&self) -> js_sys::BigInt {
        let hex = format!("{:064x}", self);
        js_sys::BigInt::from_str_radix(&hex, 16).unwrap_or_else(|_| js_sys::BigInt::from(0))
    }

    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: &str) -> Result<Uint256, JsError> {
        let clean = hex.strip_prefix("0x").unwrap_or(hex);
        if clean.len() > 64 {
            return Err(JsError::new("Hex string exceeds 256 bits"));
        }
        let mut bytes = [0u8; 32];
        let decoded = hex::decode(format!("{:0>64}", clean))
            .map_err(|e| JsError::new(&format!("Invalid hex: {}", e)))?;
        bytes.copy_from_slice(&decoded);
        Ok(Uint256::from_be_bytes(bytes))
    }
}
