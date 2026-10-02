//! Block status bitflags in the validation pipeline.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "borsh", borsh(use_discriminant = true))]
#[repr(u8)]
pub enum BlockStatus {
    #[default]
    StatusInvalid = 0x01,
    StatusHeaderOnly = 0x02,
    StatusDisqualifiedFromChain = 0x04,
    StatusUTXOValid = 0x08,
    StatusUTXOInconclusive = 0x10,
}

impl BlockStatus {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::StatusInvalid | Self::StatusDisqualifiedFromChain)
    }

    pub fn is_header_only(&self) -> bool {
        matches!(self, Self::StatusHeaderOnly)
    }

    pub fn is_utxo_valid(&self) -> bool {
        matches!(self, Self::StatusUTXOValid)
    }
}
