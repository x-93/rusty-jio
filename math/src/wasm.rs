//! WebAssembly and JavaScript bindings and conversion traits for big integers.

pub use js_sys::BigInt;
pub use wasm_bindgen::JsCast;
pub use wasm_bindgen::JsValue;
pub use workflow_core::sendable::Sendable;

pub trait JsValueExtension {
    fn try_as_vec_u8(&self) -> Result<Vec<u8>, crate::Error>;
}

impl JsValueExtension for wasm_bindgen::JsValue {
    fn try_as_vec_u8(&self) -> Result<Vec<u8>, crate::Error> {
        if let Some(s) = self.as_string() {
            let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(&s);
            let mut bytes = vec![0u8; s.len() / 2];
            faster_hex::hex_decode(s.as_bytes(), &mut bytes).map_err(crate::Error::Hex)?;
            Ok(bytes)
        } else if js_sys::Array::is_array(self) {
            let arr = js_sys::Array::from(self);
            let mut bytes = Vec::with_capacity(arr.length() as usize);
            for i in 0..arr.length() {
                bytes.push(arr.get(i).as_f64().unwrap_or(0.0) as u8);
            }
            Ok(bytes)
        } else {
            let uint8 = js_sys::Uint8Array::new(self);
            Ok(uint8.to_vec())
        }
    }
}
