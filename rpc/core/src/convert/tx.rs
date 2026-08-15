use crate::model::tx::RpcTransaction;
use jio_consensus_core::tx::Transaction;

pub fn rpc_tx_to_consensus(rpc_tx: RpcTransaction) -> Transaction {
    rpc_tx
}

pub fn consensus_tx_to_rpc(tx: Transaction) -> RpcTransaction {
    tx
}
