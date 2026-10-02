//! Sighash flags.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "borsh", borsh(use_discriminant = true))]
#[repr(u8)]
pub enum SigHashType {
    #[default]
    All = 0x01,
    None = 0x02,
    Single = 0x03,
    AnyoneCanPay = 0x80,
}

impl SigHashType {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}
