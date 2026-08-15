use crate::model::block::RpcBlock;
use jio_consensus_core::block::Block;

pub fn rpc_block_to_consensus(rpc_block: RpcBlock) -> Block {
    rpc_block
}

pub fn consensus_block_to_rpc(block: Block) -> RpcBlock {
    block
}
