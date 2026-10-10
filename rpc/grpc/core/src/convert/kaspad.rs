use crate::protowire::{jiopad_request, jiopad_response, JiopadRequest, JiopadResponse};
use jio_rpc_core::api::ops::RpcApiOps;

impl From<&jiopad_request::Payload> for RpcApiOps {
    fn from(item: &jiopad_request::Payload) -> Self {
        use jiopad_request::Payload;
        match item {
            Payload::SubmitBlockRequest(_) => RpcApiOps::SubmitBlock,
            Payload::GetBlockTemplateRequest(_) => RpcApiOps::GetBlockTemplate,
            Payload::GetCurrentNetworkRequest(_) => RpcApiOps::GetCurrentNetwork,
            Payload::GetBlockRequest(_) => RpcApiOps::GetBlock,
            Payload::GetBlocksRequest(_) => RpcApiOps::GetBlocks,
            Payload::GetInfoRequest(_) => RpcApiOps::GetInfo,

            Payload::ShutdownRequest(_) => RpcApiOps::Shutdown,
            Payload::GetPeerAddressesRequest(_) => RpcApiOps::GetPeerAddresses,
            Payload::GetSelectedTipHashRequest(_) => RpcApiOps::GetSelectedTipHash,
            Payload::GetMempoolEntryRequest(_) => RpcApiOps::GetMempoolEntry,
            Payload::GetMempoolEntriesRequest(_) => RpcApiOps::GetMempoolEntries,
            Payload::GetConnectedPeerInfoRequest(_) => RpcApiOps::GetConnectedPeerInfo,
            Payload::AddPeerRequest(_) => RpcApiOps::AddPeer,
            Payload::SubmitTransactionRequest(_) => RpcApiOps::SubmitTransaction,
            Payload::GetSubnetworkRequest(_) => RpcApiOps::GetSubnetwork,
            Payload::GetVirtualChainFromBlockRequest(_) => RpcApiOps::GetVirtualChainFromBlock,
            Payload::GetBlockCountRequest(_) => RpcApiOps::GetBlockCount,
            Payload::GetBlockDagInfoRequest(_) => RpcApiOps::GetBlockDagInfo,
            Payload::ResolveFinalityConflictRequest(_) => RpcApiOps::ResolveFinalityConflict,
            Payload::GetHeadersRequest(_) => RpcApiOps::GetHeaders,
            Payload::GetUtxosByAddressesRequest(_) => RpcApiOps::GetUtxosByAddresses,
            Payload::GetBalanceByAddressRequest(_) => RpcApiOps::GetBalanceByAddress,
            Payload::GetBalancesByAddressesRequest(_) => RpcApiOps::GetBalancesByAddresses,
            Payload::GetSinkBlueScoreRequest(_) => RpcApiOps::GetSinkBlueScore,
            Payload::BanRequest(_) => RpcApiOps::Ban,
            Payload::UnbanRequest(_) => RpcApiOps::Unban,
            Payload::EstimateNetworkHashesPerSecondRequest(_) => RpcApiOps::EstimateNetworkHashesPerSecond,
            Payload::GetMempoolEntriesByAddressesRequest(_) => RpcApiOps::GetMempoolEntriesByAddresses,
            Payload::GetCoinSupplyRequest(_) => RpcApiOps::GetCoinSupply,
            Payload::PingRequest(_) => RpcApiOps::Ping,
            Payload::GetProcessMetricsRequest(_) => RpcApiOps::GetProcessMetrics,

            // Subscription commands for starting/stopping notifications
            Payload::NotifyBlockAddedRequest(_) => RpcApiOps::NotifyBlockAdded,
            Payload::NotifyNewBlockTemplateRequest(_) => RpcApiOps::NotifyNewBlockTemplate,
            Payload::NotifyFinalityConflictRequest(_) => RpcApiOps::NotifyFinalityConflict,
            Payload::NotifyUtxosChangedRequest(_) => RpcApiOps::NotifyUtxosChanged,
            Payload::NotifySinkBlueScoreChangedRequest(_) => RpcApiOps::NotifySinkBlueScoreChanged,
            Payload::NotifyPruningPointUtxoSetOverrideRequest(_) => RpcApiOps::NotifyPruningPointUtxoSetOverride,
            Payload::NotifyVirtualDaaScoreChangedRequest(_) => RpcApiOps::NotifyVirtualDaaScoreChanged,
            Payload::NotifyVirtualChainChangedRequest(_) => RpcApiOps::NotifyVirtualChainChanged,

            Payload::StopNotifyingUtxosChangedRequest(_) => RpcApiOps::NotifyUtxosChanged,
            Payload::StopNotifyingPruningPointUtxoSetOverrideRequest(_) => RpcApiOps::NotifyPruningPointUtxoSetOverride,
        }
    }
}

impl From<&jiopad_response::Payload> for RpcApiOps {
    fn from(item: &jiopad_response::Payload) -> Self {
        use jiopad_response::Payload;
        match item {
            Payload::SubmitBlockResponse(_) => RpcApiOps::SubmitBlock,
            Payload::GetBlockTemplateResponse(_) => RpcApiOps::GetBlockTemplate,
            Payload::GetCurrentNetworkResponse(_) => RpcApiOps::GetCurrentNetwork,
            Payload::GetBlockResponse(_) => RpcApiOps::GetBlock,
            Payload::GetBlocksResponse(_) => RpcApiOps::GetBlocks,
            Payload::GetInfoResponse(_) => RpcApiOps::GetInfo,
            Payload::ShutdownResponse(_) => RpcApiOps::Shutdown,

            Payload::GetPeerAddressesResponse(_) => RpcApiOps::GetPeerAddresses,
            Payload::GetSelectedTipHashResponse(_) => RpcApiOps::GetSelectedTipHash,
            Payload::GetMempoolEntryResponse(_) => RpcApiOps::GetMempoolEntry,
            Payload::GetMempoolEntriesResponse(_) => RpcApiOps::GetMempoolEntries,
            Payload::GetConnectedPeerInfoResponse(_) => RpcApiOps::GetConnectedPeerInfo,
            Payload::AddPeerResponse(_) => RpcApiOps::AddPeer,
            Payload::SubmitTransactionResponse(_) => RpcApiOps::SubmitTransaction,
            Payload::GetSubnetworkResponse(_) => RpcApiOps::GetSubnetwork,
            Payload::GetVirtualChainFromBlockResponse(_) => RpcApiOps::GetVirtualChainFromBlock,
            Payload::GetBlockCountResponse(_) => RpcApiOps::GetBlockCount,
            Payload::GetBlockDagInfoResponse(_) => RpcApiOps::GetBlockDagInfo,
            Payload::ResolveFinalityConflictResponse(_) => RpcApiOps::ResolveFinalityConflict,
            Payload::GetHeadersResponse(_) => RpcApiOps::GetHeaders,
            Payload::GetUtxosByAddressesResponse(_) => RpcApiOps::GetUtxosByAddresses,
            Payload::GetBalanceByAddressResponse(_) => RpcApiOps::GetBalanceByAddress,
            Payload::GetBalancesByAddressesResponse(_) => RpcApiOps::GetBalancesByAddresses,
            Payload::GetSinkBlueScoreResponse(_) => RpcApiOps::GetSinkBlueScore,
            Payload::BanResponse(_) => RpcApiOps::Ban,
            Payload::UnbanResponse(_) => RpcApiOps::Unban,
            Payload::EstimateNetworkHashesPerSecondResponse(_) => RpcApiOps::EstimateNetworkHashesPerSecond,
            Payload::GetMempoolEntriesByAddressesResponse(_) => RpcApiOps::GetMempoolEntriesByAddresses,
            Payload::GetCoinSupplyResponse(_) => RpcApiOps::GetCoinSupply,
            Payload::PingResponse(_) => RpcApiOps::Ping,
            Payload::GetProcessMetricsResponse(_) => RpcApiOps::GetProcessMetrics,

            // Subscription commands for starting/stopping notifications
            Payload::NotifyBlockAddedResponse(_) => RpcApiOps::NotifyBlockAdded,
            Payload::NotifyNewBlockTemplateResponse(_) => RpcApiOps::NotifyNewBlockTemplate,
            Payload::NotifyFinalityConflictResponse(_) => RpcApiOps::NotifyFinalityConflict,
            Payload::NotifyUtxosChangedResponse(_) => RpcApiOps::NotifyUtxosChanged,
            Payload::NotifySinkBlueScoreChangedResponse(_) => RpcApiOps::NotifySinkBlueScoreChanged,
            Payload::NotifyPruningPointUtxoSetOverrideResponse(_) => RpcApiOps::NotifyPruningPointUtxoSetOverride,
            Payload::NotifyVirtualDaaScoreChangedResponse(_) => RpcApiOps::NotifyVirtualDaaScoreChanged,
            Payload::NotifyVirtualChainChangedResponse(_) => RpcApiOps::NotifyVirtualChainChanged,

            Payload::StopNotifyingPruningPointUtxoSetOverrideResponse(_) => RpcApiOps::NotifyPruningPointUtxoSetOverride,
            Payload::StopNotifyingUtxosChangedResponse(_) => RpcApiOps::NotifyUtxosChanged,

            // Notifications
            Payload::BlockAddedNotification(_) => RpcApiOps::Notification,
            Payload::NewBlockTemplateNotification(_) => RpcApiOps::Notification,
            Payload::FinalityConflictNotification(_) => RpcApiOps::Notification,
            Payload::FinalityConflictResolvedNotification(_) => RpcApiOps::Notification,
            Payload::UtxosChangedNotification(_) => RpcApiOps::Notification,
            Payload::SinkBlueScoreChangedNotification(_) => RpcApiOps::Notification,
            Payload::PruningPointUtxoSetOverrideNotification(_) => RpcApiOps::Notification,
            Payload::VirtualDaaScoreChangedNotification(_) => RpcApiOps::Notification,
            Payload::VirtualChainChangedNotification(_) => RpcApiOps::Notification,
        }
    }
}

impl From<jiopad_request::Payload> for JiopadRequest {
    fn from(item: jiopad_request::Payload) -> Self {
        JiopadRequest { id: 0, payload: Some(item) }
    }
}

impl AsRef<JiopadRequest> for JiopadRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<JiopadResponse> for JiopadResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod jiopad_request_convert {
    use crate::protowire::*;
    use jio_rpc_core::{RpcError, RpcResult};

    impl_into_jiopad_request!(Shutdown);
    impl_into_jiopad_request!(SubmitBlock);
    impl_into_jiopad_request!(GetBlockTemplate);
    impl_into_jiopad_request!(GetBlock);
    impl_into_jiopad_request!(GetInfo);

    impl_into_jiopad_request!(GetCurrentNetwork);
    impl_into_jiopad_request!(GetPeerAddresses);
    impl_into_jiopad_request!(GetSelectedTipHash);
    impl_into_jiopad_request!(GetMempoolEntry);
    impl_into_jiopad_request!(GetMempoolEntries);
    impl_into_jiopad_request!(GetConnectedPeerInfo);
    impl_into_jiopad_request!(AddPeer);
    impl_into_jiopad_request!(SubmitTransaction);
    impl_into_jiopad_request!(GetSubnetwork);
    impl_into_jiopad_request!(GetVirtualChainFromBlock);
    impl_into_jiopad_request!(GetBlocks);
    impl_into_jiopad_request!(GetBlockCount);
    impl_into_jiopad_request!(GetBlockDagInfo);
    impl_into_jiopad_request!(ResolveFinalityConflict);
    impl_into_jiopad_request!(GetHeaders);
    impl_into_jiopad_request!(GetUtxosByAddresses);
    impl_into_jiopad_request!(GetBalanceByAddress);
    impl_into_jiopad_request!(GetBalancesByAddresses);
    impl_into_jiopad_request!(GetSinkBlueScore);
    impl_into_jiopad_request!(Ban);
    impl_into_jiopad_request!(Unban);
    impl_into_jiopad_request!(EstimateNetworkHashesPerSecond);
    impl_into_jiopad_request!(GetMempoolEntriesByAddresses);
    impl_into_jiopad_request!(GetCoinSupply);
    impl_into_jiopad_request!(Ping);
    impl_into_jiopad_request!(GetProcessMetrics);

    impl_into_jiopad_request!(NotifyBlockAdded);
    impl_into_jiopad_request!(NotifyNewBlockTemplate);
    impl_into_jiopad_request!(NotifyUtxosChanged);
    impl_into_jiopad_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_jiopad_request!(NotifyFinalityConflict);
    impl_into_jiopad_request!(NotifyVirtualDaaScoreChanged);
    impl_into_jiopad_request!(NotifyVirtualChainChanged);
    impl_into_jiopad_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_jiopad_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_jiopad_request_ex!(jio_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_jiopad_request;

    macro_rules! impl_into_jiopad_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for jiopad_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for JiopadRequest {
                fn from(item: &$core_struct) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<$core_struct> for jiopad_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for JiopadRequest {
                fn from(item: $core_struct) -> Self {
                    Self { id: 0, payload: Some((&item).into()) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&jiopad_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &jiopad_request::Payload) -> RpcResult<Self> {
                    if let jiopad_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&JiopadRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &JiopadRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("JioRequest".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for JiopadRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(jiopad_request::Payload::$variant(item)) }
                }
            }

            impl From<$protowire_struct> for jiopad_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    jiopad_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_jiopad_request_ex;
}

pub mod jiopad_response_convert {
    use crate::protowire::*;
    use jio_rpc_core::{RpcError, RpcResult};

    impl_into_jiopad_response!(Shutdown);
    impl_into_jiopad_response!(SubmitBlock);
    impl_into_jiopad_response!(GetBlockTemplate);
    impl_into_jiopad_response!(GetBlock);
    impl_into_jiopad_response!(GetInfo);
    impl_into_jiopad_response!(GetCurrentNetwork);

    impl_into_jiopad_response!(GetPeerAddresses);
    impl_into_jiopad_response!(GetSelectedTipHash);
    impl_into_jiopad_response!(GetMempoolEntry);
    impl_into_jiopad_response!(GetMempoolEntries);
    impl_into_jiopad_response!(GetConnectedPeerInfo);
    impl_into_jiopad_response!(AddPeer);
    impl_into_jiopad_response!(SubmitTransaction);
    impl_into_jiopad_response!(GetSubnetwork);
    impl_into_jiopad_response!(GetVirtualChainFromBlock);
    impl_into_jiopad_response!(GetBlocks);
    impl_into_jiopad_response!(GetBlockCount);
    impl_into_jiopad_response!(GetBlockDagInfo);
    impl_into_jiopad_response!(ResolveFinalityConflict);
    impl_into_jiopad_response!(GetHeaders);
    impl_into_jiopad_response!(GetUtxosByAddresses);
    impl_into_jiopad_response!(GetBalanceByAddress);
    impl_into_jiopad_response!(GetBalancesByAddresses);
    impl_into_jiopad_response!(GetSinkBlueScore);
    impl_into_jiopad_response!(Ban);
    impl_into_jiopad_response!(Unban);
    impl_into_jiopad_response!(EstimateNetworkHashesPerSecond);
    impl_into_jiopad_response!(GetMempoolEntriesByAddresses);
    impl_into_jiopad_response!(GetCoinSupply);
    impl_into_jiopad_response!(Ping);
    impl_into_jiopad_response!(GetProcessMetrics);

    impl_into_jiopad_notify_response!(NotifyBlockAdded);
    impl_into_jiopad_notify_response!(NotifyNewBlockTemplate);
    impl_into_jiopad_notify_response!(NotifyUtxosChanged);
    impl_into_jiopad_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_jiopad_notify_response!(NotifyFinalityConflict);
    impl_into_jiopad_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_jiopad_notify_response!(NotifyVirtualChainChanged);
    impl_into_jiopad_notify_response!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_jiopad_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_jiopad_response_ex!(jio_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
    }
    use impl_into_jiopad_response;

    macro_rules! impl_into_jiopad_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for jiopad_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    jiopad_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for JiopadResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for jiopad_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    jiopad_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for JiopadResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for jiopad_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    jiopad_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for JiopadResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(jiopad_response::Payload::$variant(item)) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&jiopad_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &jiopad_response::Payload) -> RpcResult<Self> {
                    if let jiopad_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&JiopadResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &JiopadResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("JioResponse".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_jiopad_response_ex;

    macro_rules! impl_into_jiopad_notify_response {
        ($name:tt) => {
            impl_into_jiopad_response!($name);

            paste::paste! {
                impl_into_jiopad_notify_response_ex!(jio_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
    }
    use impl_into_jiopad_notify_response;

    macro_rules! impl_into_jiopad_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_jiopad_notify_response_ex;
}
