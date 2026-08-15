use crate::converter::RpcConverter;
use async_trait::async_trait;
use jio_consensusmanager::ConsensusManager;
use jio_mining::MiningManager;
use jio_notify::Notifier;
use jio_rpc_core::api::rpc::{RpcApi, RpcError, RpcResult};
use jio_rpc_core::model::message::*;
use jio_utxoindex::UtxoIndex;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RpcCoreService {
    #[allow(dead_code)]
    consensus_manager: ConsensusManager,
    mining_manager: MiningManager,
    utxo_index: Arc<UtxoIndex>,
    #[allow(dead_code)]
    notifier: Arc<Notifier>,
}

impl RpcCoreService {
    pub fn new(
        consensus_manager: ConsensusManager,
        mining_manager: MiningManager,
        utxo_index: Arc<UtxoIndex>,
        notifier: Arc<Notifier>,
    ) -> Self {
        Self {
            consensus_manager,
            mining_manager,
            utxo_index,
            notifier,
        }
    }
}

#[async_trait]
impl RpcApi for RpcCoreService {
    async fn ping(&self) -> RpcResult<()> {
        Ok(())
    }

    async fn get_info(&self) -> RpcResult<GetInfoResponse> {
        let mempool_len = self.mining_manager.mempool().len() as u64;
        Ok(RpcConverter::build_info(mempool_len, true))
    }

    async fn get_current_network(&self) -> RpcResult<GetCurrentNetworkResponse> {
        Ok(GetCurrentNetworkResponse {
            network: jio_consensus_core::network::NetworkType::Devnet,
        })
    }

    async fn submit_block(&self, request: SubmitBlockRequest) -> RpcResult<SubmitBlockResponse> {
        let block_arc = Arc::new(request.block.clone());
        let hash = self
            .mining_manager
            .submit_block(request.block)
            .map_err(|e| RpcError::General(e.to_string()))?;

        // 1. Broadcast BlockAdded notification
        self.notifier
            .notify(jio_notify::events::Notification::BlockAdded {
                block: block_arc.clone(),
            })
            .await;

        // 2. Index block UTXOs
        let mut diff = jio_consensus_core::utxo::UtxoDiff::default();
        for tx in &block_arc.transactions {
            let tx_id = tx.id();
            for (idx, out) in tx.outputs.iter().enumerate() {
                let outpoint = jio_consensus_core::tx::TransactionOutpoint::new(tx_id, idx as u32);
                let entry = jio_consensus_core::utxo::UtxoEntry::new(
                    out.value,
                    out.script_public_key.clone(),
                    block_arc.header.daa_score,
                    tx.is_coinbase(),
                );
                diff.to_add.insert(outpoint, entry);
            }
        }
        self.utxo_index.update(&diff);

        Ok(SubmitBlockResponse { hash })
    }

    async fn get_block_template(&self, request: GetBlockTemplateRequest) -> RpcResult<GetBlockTemplateResponse> {
        let template = self
            .mining_manager
            .get_block_template(request.payee_script_public_key, request.extra_data)
            .map_err(|e| RpcError::General(e.to_string()))?;
        Ok(GetBlockTemplateResponse {
            block: template.block,
            is_synced: template.is_synced,
        })
    }

    async fn submit_transaction(&self, request: SubmitTransactionRequest) -> RpcResult<SubmitTransactionResponse> {
        let candidate = self
            .mining_manager
            .validate_and_insert_transaction(Arc::new(request.transaction))
            .map_err(|e| RpcError::General(e.to_string()))?;
        Ok(SubmitTransactionResponse {
            transaction_id: candidate.id(),
        })
    }

    async fn get_utxos_by_addresses(&self, request: GetUtxosByAddressesRequest) -> RpcResult<GetUtxosByAddressesResponse> {
        use jio_utxoindex::core::UtxoIndexReader;
        let mut entries = HashMap::new();
        for addr in request.addresses {
            if let Some(utxos) = self.utxo_index.get_utxos_by_script_public_key(&addr) {
                for (op, entry) in utxos {
                    entries.insert(op, entry);
                }
            }
        }
        Ok(GetUtxosByAddressesResponse { entries })
    }

    async fn get_balance_by_address(&self, request: GetBalanceByAddressRequest) -> RpcResult<GetBalanceByAddressResponse> {
        use jio_utxoindex::core::UtxoIndexReader;
        let mut total = 0u64;
        if let Some(utxos) = self.utxo_index.get_utxos_by_script_public_key(&request.address) {
            for entry in utxos.values() {
                total += entry.amount;
            }
        }
        Ok(GetBalanceByAddressResponse { balance: total })
    }
}
