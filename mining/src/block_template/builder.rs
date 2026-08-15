use crate::block_template::selector::TransactionSelector;
use crate::mempool::Mempool;
use jio_consensus::consensus::ctl::ConsensusCtl;
use jio_consensus::processes::coinbase::serialize_coinbase_payload;
use jio_consensus_core::block::Block;
use jio_consensus_core::constants::{SOMPI_PER_JIO, TX_VERSION};
use jio_consensus_core::header::Header;
use jio_consensus_core::merkle::calc_tx_merkle_root;
use jio_consensus_core::subnets::SUBNETWORK_ID_COINBASE;
use jio_consensus_core::tx::{
    ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput,
};
use jio_hashes::Hash;
use jio_mining_errors::{MiningError, MiningResult};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BlockTemplate {
    pub block: Block,
    pub is_synced: bool,
}

pub struct BlockTemplateBuilder;

impl BlockTemplateBuilder {
    pub fn build_block_template(
        consensus: &Arc<dyn ConsensusCtl>,
        mempool: &Mempool,
        payee_script_public_key: ScriptPublicKey,
        extra_data: Vec<u8>,
    ) -> MiningResult<BlockTemplate> {
        let virtual_state = consensus
            .get_virtual_state()
            .ok_or_else(|| MiningError::BlockCreation("virtual state not initialized".to_string()))?;

        let candidates = mempool.get_all_candidates();
        let (selected_txs, total_fees, _total_mass) =
            TransactionSelector::select_transactions(candidates, 500_000);

        let now = jio_core::time::unix_now();
        let timestamp = now.max(virtual_state.past_median_time + 1);

        // Base reward calculation: 50 JIO base subsidy
        let base_subsidy = 50 * SOMPI_PER_JIO;
        let coinbase_value = base_subsidy + total_fees;

        let mut payload_extra = extra_data;
        payload_extra.extend_from_slice(&timestamp.to_le_bytes());

        let payload = serialize_coinbase_payload(
            virtual_state.blue_score,
            &payee_script_public_key,
            &payload_extra,
        );

        let coinbase_tx = Transaction {
            version: TX_VERSION,
            inputs: vec![TransactionInput {
                previous_outpoint: TransactionOutpoint::new(Hash::default(), u32::MAX),
                signature_script: Vec::new(),
                sequence: 0,
                sig_op_count: 0,
            }],
            outputs: vec![TransactionOutput::new(
                coinbase_value,
                payee_script_public_key.clone(),
            )],
            lock_time: 0,
            subnetwork_id: SUBNETWORK_ID_COINBASE,
            gas: 0,
            payload,
            mass: 0,
        };

        let mut block_transactions = Vec::with_capacity(1 + selected_txs.len());
        block_transactions.push(coinbase_tx);
        for tx in selected_txs {
            block_transactions.push((*tx).clone());
        }

        let hash_merkle_root = calc_tx_merkle_root(&block_transactions);

        let header = Header::new_finalized(
            1, // version
            vec![virtual_state.parents.clone()],
            hash_merkle_root,
            Hash::from_bytes([0u8; 32]), // accepted_id_merkle_root
            virtual_state.utxo_commitment,
            timestamp,
            virtual_state.bits,
            0, // nonce initially 0
            virtual_state.daa_score,
            jio_consensus_core::BlueWorkType::from_u64(virtual_state.blue_score),
            virtual_state.blue_score,
            Hash::from_bytes([0u8; 32]), // pruning point
        );

        let block = Block::new(header, block_transactions);
        Ok(BlockTemplate {
            block,
            is_synced: true,
        })
    }
}
