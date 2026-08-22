use crate::pb::jiopad_message::Payload as JiopadMessagePayload;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum JiopadMessagePayloadType {
    Addresses = 0,
    Block,
    Transaction,
    BlockLocator,
    RequestAddresses,
    RequestRelayBlocks,
    RequestTransactions,
    IbdBlock,
    InvRelayBlock,
    InvTransactions,
    Ping,
    Pong,
    Verack,
    Version,
    TransactionNotFound,
    Reject,
    PruningPointUtxoSetChunk,
    RequestIbdBlocks,
    UnexpectedPruningPoint,
    IbdBlockLocator,
    IbdBlockLocatorHighestHash,
    RequestNextPruningPointUtxoSetChunk,
    DonePruningPointUtxoSetChunks,
    IbdBlockLocatorHighestHashNotFound,
    BlockWithTrustedData,
    DoneBlocksWithTrustedData,
    RequestPruningPointAndItsAnticone,
    BlockHeaders,
    RequestNextHeaders,
    DoneHeaders,
    RequestPruningPointUtxoSet,
    RequestHeaders,
    RequestBlockLocator,
    PruningPoints,
    RequestPruningPointProof,
    PruningPointProof,
    Ready,
    BlockWithTrustedDataV4,
    TrustedData,
    RequestIbdChainBlockLocator,
    IbdChainBlockLocator,
    RequestAntipast,
    RequestNextPruningPointAndItsAnticoneBlocks,
}

impl From<&JiopadMessagePayload> for JiopadMessagePayloadType {
    fn from(payload: &JiopadMessagePayload) -> Self {
        match payload {
            JiopadMessagePayload::Addresses(_) => JiopadMessagePayloadType::Addresses,
            JiopadMessagePayload::Block(_) => JiopadMessagePayloadType::Block,
            JiopadMessagePayload::Transaction(_) => JiopadMessagePayloadType::Transaction,
            JiopadMessagePayload::BlockLocator(_) => JiopadMessagePayloadType::BlockLocator,
            JiopadMessagePayload::RequestAddresses(_) => JiopadMessagePayloadType::RequestAddresses,
            JiopadMessagePayload::RequestRelayBlocks(_) => JiopadMessagePayloadType::RequestRelayBlocks,
            JiopadMessagePayload::RequestTransactions(_) => JiopadMessagePayloadType::RequestTransactions,
            JiopadMessagePayload::IbdBlock(_) => JiopadMessagePayloadType::IbdBlock,
            JiopadMessagePayload::InvRelayBlock(_) => JiopadMessagePayloadType::InvRelayBlock,
            JiopadMessagePayload::InvTransactions(_) => JiopadMessagePayloadType::InvTransactions,
            JiopadMessagePayload::Ping(_) => JiopadMessagePayloadType::Ping,
            JiopadMessagePayload::Pong(_) => JiopadMessagePayloadType::Pong,
            JiopadMessagePayload::Verack(_) => JiopadMessagePayloadType::Verack,
            JiopadMessagePayload::Version(_) => JiopadMessagePayloadType::Version,
            JiopadMessagePayload::TransactionNotFound(_) => JiopadMessagePayloadType::TransactionNotFound,
            JiopadMessagePayload::Reject(_) => JiopadMessagePayloadType::Reject,
            JiopadMessagePayload::PruningPointUtxoSetChunk(_) => JiopadMessagePayloadType::PruningPointUtxoSetChunk,
            JiopadMessagePayload::RequestIbdBlocks(_) => JiopadMessagePayloadType::RequestIbdBlocks,
            JiopadMessagePayload::UnexpectedPruningPoint(_) => JiopadMessagePayloadType::UnexpectedPruningPoint,
            JiopadMessagePayload::IbdBlockLocator(_) => JiopadMessagePayloadType::IbdBlockLocator,
            JiopadMessagePayload::IbdBlockLocatorHighestHash(_) => JiopadMessagePayloadType::IbdBlockLocatorHighestHash,
            JiopadMessagePayload::RequestNextPruningPointUtxoSetChunk(_) => {
                JiopadMessagePayloadType::RequestNextPruningPointUtxoSetChunk
            }
            JiopadMessagePayload::DonePruningPointUtxoSetChunks(_) => JiopadMessagePayloadType::DonePruningPointUtxoSetChunks,
            JiopadMessagePayload::IbdBlockLocatorHighestHashNotFound(_) => {
                JiopadMessagePayloadType::IbdBlockLocatorHighestHashNotFound
            }
            JiopadMessagePayload::BlockWithTrustedData(_) => JiopadMessagePayloadType::BlockWithTrustedData,
            JiopadMessagePayload::DoneBlocksWithTrustedData(_) => JiopadMessagePayloadType::DoneBlocksWithTrustedData,
            JiopadMessagePayload::RequestPruningPointAndItsAnticone(_) => JiopadMessagePayloadType::RequestPruningPointAndItsAnticone,
            JiopadMessagePayload::BlockHeaders(_) => JiopadMessagePayloadType::BlockHeaders,
            JiopadMessagePayload::RequestNextHeaders(_) => JiopadMessagePayloadType::RequestNextHeaders,
            JiopadMessagePayload::DoneHeaders(_) => JiopadMessagePayloadType::DoneHeaders,
            JiopadMessagePayload::RequestPruningPointUtxoSet(_) => JiopadMessagePayloadType::RequestPruningPointUtxoSet,
            JiopadMessagePayload::RequestHeaders(_) => JiopadMessagePayloadType::RequestHeaders,
            JiopadMessagePayload::RequestBlockLocator(_) => JiopadMessagePayloadType::RequestBlockLocator,
            JiopadMessagePayload::PruningPoints(_) => JiopadMessagePayloadType::PruningPoints,
            JiopadMessagePayload::RequestPruningPointProof(_) => JiopadMessagePayloadType::RequestPruningPointProof,
            JiopadMessagePayload::PruningPointProof(_) => JiopadMessagePayloadType::PruningPointProof,
            JiopadMessagePayload::Ready(_) => JiopadMessagePayloadType::Ready,
            JiopadMessagePayload::BlockWithTrustedDataV4(_) => JiopadMessagePayloadType::BlockWithTrustedDataV4,
            JiopadMessagePayload::TrustedData(_) => JiopadMessagePayloadType::TrustedData,
            JiopadMessagePayload::RequestIbdChainBlockLocator(_) => JiopadMessagePayloadType::RequestIbdChainBlockLocator,
            JiopadMessagePayload::IbdChainBlockLocator(_) => JiopadMessagePayloadType::IbdChainBlockLocator,
            JiopadMessagePayload::RequestAntipast(_) => JiopadMessagePayloadType::RequestAntipast,
            JiopadMessagePayload::RequestNextPruningPointAndItsAnticoneBlocks(_) => {
                JiopadMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks
            }
        }
    }
}
