use jio_bip32::child_number::ChildNumber;

pub const JIO_COIN_TYPE: u32 = 111111;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivationPath {
    pub account: u32,
    pub is_change: bool,
    pub address_index: u32,
}

impl DerivationPath {
    pub fn new(account: u32, is_change: bool, address_index: u32) -> Self {
        Self {
            account,
            is_change,
            address_index,
        }
    }

    pub fn to_child_numbers(&self) -> Vec<ChildNumber> {
        vec![
            ChildNumber::hardened(44),
            ChildNumber::hardened(JIO_COIN_TYPE),
            ChildNumber::hardened(self.account),
            ChildNumber::normal(if self.is_change { 1 } else { 0 }),
            ChildNumber::normal(self.address_index),
        ]
    }
}
