use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::utxo::UtxoDiff;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    BlockAdded,
    VirtualSelectedParentChainChanged,
    FinalityConflict,
    UtxosChanged,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Notification {
    BlockAdded {
        block: Arc<Block>,
    },
    VirtualSelectedParentChainChanged {
        removed_chain_block_hashes: Vec<BlockHash>,
        added_chain_block_hashes: Vec<BlockHash>,
    },
    FinalityConflict {
        violating_block_hash: BlockHash,
    },
    UtxosChanged {
        utxo_diff: Arc<UtxoDiff>,
    },
    SinkBlueScoreChanged {
        sink_blue_score: u64,
    },
    VirtualDaaScoreChanged {
        virtual_daa_score: u64,
    },
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
}

impl Notification {
    pub fn event_type(&self) -> EventType {
        match self {
            Notification::BlockAdded { .. } => EventType::BlockAdded,
            Notification::VirtualSelectedParentChainChanged { .. } => {
                EventType::VirtualSelectedParentChainChanged
            }
            Notification::FinalityConflict { .. } => EventType::FinalityConflict,
            Notification::UtxosChanged { .. } => EventType::UtxosChanged,
            Notification::SinkBlueScoreChanged { .. } => EventType::SinkBlueScoreChanged,
            Notification::VirtualDaaScoreChanged { .. } => EventType::VirtualDaaScoreChanged,
            Notification::PruningPointUtxoSetOverride => EventType::PruningPointUtxoSetOverride,
            Notification::NewBlockTemplate => EventType::NewBlockTemplate,
        }
    }
}
