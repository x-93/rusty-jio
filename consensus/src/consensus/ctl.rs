use crate::model::stores::virtual_state::VirtualState;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::errors::consensus::ConsensusResult;
use jio_consensus_core::header::Header;
use jio_consensus_core::tx::TransactionOutpoint;
use jio_consensus_core::utxo::UtxoEntry;
use std::sync::Arc;

pub trait ConsensusCtl: Send + Sync {
    fn validate_and_insert_block(&self, block: Block) -> ConsensusResult<BlockHash>;
    fn validate_and_insert_header(&self, header: &Header) -> ConsensusResult<BlockHash>;
    fn get_virtual_state(&self) -> Option<Arc<VirtualState>>;
    fn get_header(&self, hash: &BlockHash) -> Option<Arc<Header>>;
    fn get_status(&self, hash: &BlockHash) -> Option<BlockStatus>;
    fn get_utxo(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry>;
    fn get_selected_chain_tip(&self) -> Option<BlockHash>;
}
