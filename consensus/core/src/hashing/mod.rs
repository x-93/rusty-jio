pub mod header;
pub mod sighash;
pub mod sighash_type;
pub mod tx;
pub mod wasm;

use jio_hashes::HasherBase;

pub trait HasherExtensions {
    fn write_bool(&mut self, val: bool) -> &mut Self;
    fn write_var_bytes(&mut self, bytes: &[u8]) -> &mut Self;
    fn write_u16(&mut self, val: u16) -> &mut Self;
    fn write_u32(&mut self, val: u32) -> &mut Self;
    fn write_u64(&mut self, val: u64) -> &mut Self;
}

impl<T: HasherBase> HasherExtensions for T {
    #[inline(always)]
    fn write_bool(&mut self, val: bool) -> &mut Self {
        self.update([if val { 1u8 } else { 0u8 }])
    }

    #[inline(always)]
    fn write_var_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.update((bytes.len() as u64).to_le_bytes());
        self.update(bytes)
    }

    #[inline(always)]
    fn write_u16(&mut self, val: u16) -> &mut Self {
        self.update(val.to_le_bytes())
    }

    #[inline(always)]
    fn write_u32(&mut self, val: u32) -> &mut Self {
        self.update(val.to_le_bytes())
    }

    #[inline(always)]
    fn write_u64(&mut self, val: u64) -> &mut Self {
        self.update(val.to_le_bytes())
    }
}