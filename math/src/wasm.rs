use crate::uint::Uint256;

pub fn uint256_to_hex_string(val: &Uint256) -> String {
    format!("{:x}", val)
}

pub fn uint256_from_hex_str(s: &str) -> Result<Uint256, String> {
    use jio_utils::hex::FromHex;
    let bytes = Vec::<u8>::from_hex(s).map_err(|e| e.to_string())?;
    if bytes.len() > 32 {
        return Err("hex too long for 256-bit uint".to_string());
    }
    let mut fixed = [0u8; 32];
    let offset = 32 - bytes.len();
    fixed[offset..].copy_from_slice(&bytes);
    Ok(Uint256::from_be_bytes(fixed))
}
