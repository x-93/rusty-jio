use crate::model::tx::RpcUtxoEntry;
use jio_consensus_core::utxo::UtxoEntry;

pub fn rpc_utxo_to_consensus(rpc_utxo: RpcUtxoEntry) -> UtxoEntry {
    rpc_utxo
}

pub fn consensus_utxo_to_rpc(utxo: UtxoEntry) -> RpcUtxoEntry {
    utxo
}
