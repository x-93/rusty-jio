use crate::block::Block;
use crate::header::Header;
use jio_hashes::Hash;

#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub header: Header,
}

impl GenesisBlock {
    pub const fn new(header: Header) -> Self {
        Self { header }
    }

    pub fn to_block(&self) -> Block {
        Block::from_header(self.header.clone())
    }
}

pub fn create_genesis_header(version: u16, timestamp: u64, bits: u32) -> Header {
    Header::new(
        version,
        vec![vec![]], // Genesis has no parents
        Hash::from_bytes([0u8; 32]),
        Hash::from_bytes([0u8; 32]),
        Hash::from_bytes([0u8; 32]),
        timestamp,
        bits,
        0,
        0,
        0.into(),
        0,
        Hash::from_bytes([0u8; 32]),
    )
}
