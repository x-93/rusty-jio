//! Hardcoded genesis block constants for all network configurations.

use super::super::block::Block;
use super::super::header::Header;
use super::super::network::NetworkType;
use jio_hashes::{Hash, ZERO_HASH};

#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub block: Block,
}

pub fn get_genesis(net: NetworkType) -> GenesisBlock {
    let timestamp = match net {
        NetworkType::Mainnet => 1700000000000,
        _ => 1700000001000,
    };
    let header = Header::new_finalized(
        1,
        Vec::new(),
        ZERO_HASH,
        ZERO_HASH,
        ZERO_HASH,
        timestamp,
        0x1e7fffff,
        0,
        0,
        0.into(),
        0,
        ZERO_HASH,
    );
    let hash = header.hash;
    GenesisBlock {
        hash,
        block: Block::new(header, Vec::new()),
    }
}
