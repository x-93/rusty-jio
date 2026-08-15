use crate::events::EventType;
use jio_consensus_core::tx::ScriptPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    BlockAdded,
    VirtualSelectedParentChainChanged { include_accepted_transaction_ids: bool },
    FinalityConflict,
    UtxosChanged { addresses: Vec<ScriptPublicKey> },
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
}

impl Scope {
    pub fn event_type(&self) -> EventType {
        match self {
            Scope::BlockAdded => EventType::BlockAdded,
            Scope::VirtualSelectedParentChainChanged { .. } => {
                EventType::VirtualSelectedParentChainChanged
            }
            Scope::FinalityConflict => EventType::FinalityConflict,
            Scope::UtxosChanged { .. } => EventType::UtxosChanged,
            Scope::SinkBlueScoreChanged => EventType::SinkBlueScoreChanged,
            Scope::VirtualDaaScoreChanged => EventType::VirtualDaaScoreChanged,
            Scope::PruningPointUtxoSetOverride => EventType::PruningPointUtxoSetOverride,
            Scope::NewBlockTemplate => EventType::NewBlockTemplate,
        }
    }
}
