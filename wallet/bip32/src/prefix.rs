#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prefix {
    pub bytes: [u8; 4],
}

pub const XPUB_MAINNET: Prefix = Prefix {
    bytes: [0x04, 0x88, 0xB2, 0x1E],
};
pub const XPRV_MAINNET: Prefix = Prefix {
    bytes: [0x04, 0x88, 0xAD, 0xE4],
};
