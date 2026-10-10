use self::{
    address::{ReceiveAddressesFlow, SendAddressesFlow},
    blockrelay::{flow::HandleRelayInvsFlow, handle_requests::HandleRelayBlockRequests},
    ibd::IbdFlow,
    ping::{ReceivePingsFlow, SendPingsFlow},
    request_anticone::HandleAnticoneRequests,
    request_block_locator::RequestBlockLocatorFlow,
    request_headers::RequestHeadersFlow,
    request_ibd_blocks::HandleIbdBlockRequests,
    request_ibd_chain_block_locator::RequestIbdChainBlockLocatorFlow,
    request_pp_proof::RequestPruningPointProofFlow,
    request_pruning_point_and_anticone::PruningPointAndItsAnticoneRequestsFlow,
    request_pruning_point_utxo_set::RequestPruningPointUtxoSetFlow,
    txrelay::flow::{RelayTransactionsFlow, RequestTransactionsFlow},
};
use crate::{flow_context::FlowContext, flow_trait::Flow};

use jio_p2p_lib::{pb::jiopad_message::Payload as JiopadMessagePayload, JiopadMessagePayloadType, Router};
use log::{debug, warn};
use std::sync::Arc;

mod address;
mod blockrelay;
mod ibd;
mod ping;
mod request_anticone;
mod request_block_locator;
mod request_headers;
mod request_ibd_blocks;
mod request_ibd_chain_block_locator;
mod request_pp_proof;
mod request_pruning_point_and_anticone;
mod request_pruning_point_utxo_set;
mod txrelay;

pub fn register(ctx: FlowContext, router: Arc<Router>) -> Vec<Box<dyn Flow>> {
    // IBD flow <-> invs flow channel requires no buffering hence the minimal size possible
    let (ibd_sender, relay_receiver) = tokio::sync::mpsc::channel(1);
    let flows: Vec<Box<dyn Flow>> = vec![
        Box::new(IbdFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![
                JiopadMessagePayloadType::BlockHeaders,
                JiopadMessagePayloadType::DoneHeaders,
                JiopadMessagePayloadType::IbdBlockLocatorHighestHash,
                JiopadMessagePayloadType::IbdBlockLocatorHighestHashNotFound,
                JiopadMessagePayloadType::BlockWithTrustedDataV4,
                JiopadMessagePayloadType::DoneBlocksWithTrustedData,
                JiopadMessagePayloadType::IbdChainBlockLocator,
                JiopadMessagePayloadType::IbdBlock,
                JiopadMessagePayloadType::TrustedData,
                JiopadMessagePayloadType::PruningPoints,
                JiopadMessagePayloadType::PruningPointProof,
                JiopadMessagePayloadType::UnexpectedPruningPoint,
                JiopadMessagePayloadType::PruningPointUtxoSetChunk,
                JiopadMessagePayloadType::DonePruningPointUtxoSetChunks,
            ]),
            relay_receiver,
        )),
        Box::new(HandleRelayInvsFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::InvRelayBlock]),
            router.subscribe(vec![JiopadMessagePayloadType::Block, JiopadMessagePayloadType::BlockLocator]),
            ibd_sender,
        )),
        Box::new(HandleRelayBlockRequests::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestRelayBlocks]),
        )),
        Box::new(ReceivePingsFlow::new(ctx.clone(), router.clone(), router.subscribe(vec![JiopadMessagePayloadType::Ping]))),
        Box::new(SendPingsFlow::new(ctx.clone(), Arc::downgrade(&router), router.subscribe(vec![JiopadMessagePayloadType::Pong]))),
        Box::new(RequestHeadersFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestHeaders, JiopadMessagePayloadType::RequestNextHeaders]),
        )),
        Box::new(RequestPruningPointProofFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestPruningPointProof]),
        )),
        Box::new(RequestIbdChainBlockLocatorFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestIbdChainBlockLocator]),
        )),
        Box::new(PruningPointAndItsAnticoneRequestsFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![
                JiopadMessagePayloadType::RequestPruningPointAndItsAnticone,
                JiopadMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks,
            ]),
        )),
        Box::new(RequestPruningPointUtxoSetFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![
                JiopadMessagePayloadType::RequestPruningPointUtxoSet,
                JiopadMessagePayloadType::RequestNextPruningPointUtxoSetChunk,
            ]),
        )),
        Box::new(HandleIbdBlockRequests::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestIbdBlocks]),
        )),
        Box::new(HandleAnticoneRequests::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestAnticone]),
        )),
        Box::new(RelayTransactionsFlow::new(
            ctx.clone(),
            router.clone(),
            router
                .subscribe_with_capacity(vec![JiopadMessagePayloadType::InvTransactions], RelayTransactionsFlow::invs_channel_size()),
            router.subscribe(vec![JiopadMessagePayloadType::Transaction, JiopadMessagePayloadType::TransactionNotFound]),
        )),
        Box::new(RequestTransactionsFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestTransactions]),
        )),
        Box::new(ReceiveAddressesFlow::new(ctx.clone(), router.clone(), router.subscribe(vec![JiopadMessagePayloadType::Addresses]))),
        Box::new(SendAddressesFlow::new(
            ctx.clone(),
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestAddresses]),
        )),
        Box::new(RequestBlockLocatorFlow::new(
            ctx,
            router.clone(),
            router.subscribe(vec![JiopadMessagePayloadType::RequestBlockLocator]),
        )),
    ];

    // TEMP: subscribe to remaining messages and ignore them
    // NOTE: as flows are implemented, the below types should be all commented out
    let mut unimplemented_messages_route = router.subscribe(vec![
        // JiopadMessagePayloadType::Addresses,
        // JiopadMessagePayloadType::Block,
        // JiopadMessagePayloadType::Transaction,
        // JiopadMessagePayloadType::BlockLocator,
        // JiopadMessagePayloadType::RequestAddresses,
        // JiopadMessagePayloadType::RequestRelayBlocks,
        // JiopadMessagePayloadType::RequestTransactions,
        // JiopadMessagePayloadType::IbdBlock,
        // JiopadMessagePayloadType::InvRelayBlock,
        // JiopadMessagePayloadType::InvTransactions,
        // JiopadMessagePayloadType::Ping,
        // JiopadMessagePayloadType::Pong,
        // JiopadMessagePayloadType::Verack,
        // JiopadMessagePayloadType::Version,
        // JiopadMessagePayloadType::Ready,
        // JiopadMessagePayloadType::TransactionNotFound,
        JiopadMessagePayloadType::Reject,
        // JiopadMessagePayloadType::PruningPointUtxoSetChunk,
        // JiopadMessagePayloadType::RequestIbdBlocks,
        // JiopadMessagePayloadType::UnexpectedPruningPoint,
        // JiopadMessagePayloadType::IbdBlockLocatorHighestHash,
        // JiopadMessagePayloadType::RequestNextPruningPointUtxoSetChunk,
        // JiopadMessagePayloadType::DonePruningPointUtxoSetChunks,
        // JiopadMessagePayloadType::IbdBlockLocatorHighestHashNotFound,

        // We do not register the below two messages since they are deprecated also in go-jio
        // JiopadMessagePayloadType::BlockWithTrustedData,
        // JiopadMessagePayloadType::IbdBlockLocator,

        // JiopadMessagePayloadType::DoneBlocksWithTrustedData,
        // JiopadMessagePayloadType::RequestPruningPointAndItsAnticone,
        // JiopadMessagePayloadType::BlockHeaders,
        // JiopadMessagePayloadType::RequestNextHeaders,
        // JiopadMessagePayloadType::DoneHeaders,
        // JiopadMessagePayloadType::RequestPruningPointUtxoSet,
        // JiopadMessagePayloadType::RequestHeaders,
        // JiopadMessagePayloadType::RequestBlockLocator,
        // JiopadMessagePayloadType::PruningPoints,
        // JiopadMessagePayloadType::RequestPruningPointProof,
        // JiopadMessagePayloadType::PruningPointProof,
        // JiopadMessagePayloadType::BlockWithTrustedDataV4,
        // JiopadMessagePayloadType::TrustedData,
        // JiopadMessagePayloadType::RequestIbdChainBlockLocator,
        // JiopadMessagePayloadType::IbdChainBlockLocator,
        // JiopadMessagePayloadType::RequestAnticone,
        // JiopadMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks,
    ]);

    tokio::spawn(async move {
        while let Some(msg) = unimplemented_messages_route.recv().await {
            match msg.payload {
                Some(JiopadMessagePayload::Reject(reject_msg)) => {
                    warn!("Got a reject message {} from peer {}", reject_msg.reason, router);
                }
                _ => debug!("P2P unimplemented routes message: {:?}", msg),
            }
        }
    });

    flows
}
