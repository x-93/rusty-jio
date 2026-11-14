pub mod errors;
pub mod utxo_validation;

use crate::model::stores::statuses::StatusesStore;
use crate::model::stores::utxo_diffs::UtxoDiffsStore;
use consensus_core::block::Block;
use consensus_core::blockstatus::BlockStatus;
use consensus_core::muhash::apply_diff_to_muhash;
use consensus_core::utxo::utxo_collection::UtxoCollection;
use errors::{VirtualResult, VirtualStateError};
use jio_database::prelude::DbWriter;
use jio_hashes::Hash;
use jio_muhash::MuHash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct VirtualStateProcessor<TUtxoDiffsStore: UtxoDiffsStore, TStatusesStore: StatusesStore> {
    utxo_diffs_store: Arc<TUtxoDiffsStore>,
    statuses_store: Arc<RwLock<TStatusesStore>>,
    utxo_set: Arc<RwLock<UtxoCollection>>,
    muhash: Arc<RwLock<MuHash>>,
    coinbase_maturity: u64,
}

impl<TUtxoDiffsStore: UtxoDiffsStore, TStatusesStore: StatusesStore>
    VirtualStateProcessor<TUtxoDiffsStore, TStatusesStore>
{
    pub fn new(
        utxo_diffs_store: Arc<TUtxoDiffsStore>,
        statuses_store: Arc<RwLock<TStatusesStore>>,
        coinbase_maturity: u64,
    ) -> Self {
        Self {
            utxo_diffs_store,
            statuses_store,
            utxo_set: Arc::new(RwLock::new(UtxoCollection::new())),
            muhash: Arc::new(RwLock::new(MuHash::new())),
            coinbase_maturity,
        }
    }

    /// Processes block transactions against the current virtual UTXO set,
    /// generating the UTXO diff, updating the rolling MuHash commitment, and marking the block status.
    pub fn process_block(&self, writer: &mut impl DbWriter, block: &Block) -> VirtualResult<Hash> {
        let block_hash = block.header.hash;
        let mut utxo_set = self.utxo_set.write();

        // 1. Validate transactions and construct UTXO diff
        let (diff, _) = utxo_validation::process_block_transactions_and_build_diff(
            &block.transactions,
            &mut utxo_set,
            block.header.daa_score,
            self.coinbase_maturity,
        )?;

        // 2. Update rolling MuHash commitment
        let mut muhash = self.muhash.write();
        apply_diff_to_muhash(&mut muhash, &diff);
        let commitment = muhash.finalize();

        // 3. Persist UTXO diff in storage
        self.utxo_diffs_store
            .set_utxo_diff(writer, block_hash, Arc::new(diff))
            .map_err(VirtualStateError::Store)?;

        // 4. Update block status to StatusUTXOValid
        self.statuses_store
            .write()
            .set(writer, block_hash, BlockStatus::STATUS_UTXO_VALID)
            .map_err(VirtualStateError::Store)?;

        Ok(commitment)
    }

    /// Returns the current size of the virtual UTXO set.
    pub fn utxo_set_len(&self) -> usize {
        self.utxo_set.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::statuses::DbStatusesStore;
    use crate::model::stores::utxo_diffs::DbUtxoDiffsStore;
    use consensus_core::BlueWorkType;
    use consensus_core::header::Header;
    use consensus_core::subnets::SubnetworkId;
    use consensus_core::tx::{
        ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput,
    };
    use jio_database::prelude::{BatchDbWriter, DB};
    use tempfile::TempDir;

    #[test]
    fn test_virtual_state_processing_coinbase_and_spend() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(DB::open_default(temp_dir.path()).unwrap());
        let mut writer = BatchDbWriter::new(&db);

        let utxo_diffs_store = Arc::new(DbUtxoDiffsStore::new(db.clone(), 100));
        let statuses_store = Arc::new(RwLock::new(DbStatusesStore::new(db.clone(), 100)));

        let processor = VirtualStateProcessor::new(
            utxo_diffs_store.clone(),
            statuses_store.clone(),
            10, // Coinbase maturity = 10
        );

        // 1. Process block 1 with Coinbase producing output (outpoint: [tx1_id, 0])
        // Use OP_TRUE for the locking script so any spend without complex signatures succeeds
        let true_spk = ScriptPublicKey::from_vec(0, vec![jio_txscript::op::OP_TRUE]);
        let cb_output = TransactionOutput::new(1_000_000, true_spk);
        let cb_tx = Transaction::new(
            1,
            vec![],
            vec![cb_output],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![],
        );
        let tx1_id = cb_tx.id();

        let header1 = Header::new_finalized(
            1,
            vec![],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1000,
            0x1e7fffff,
            0,
            1,
            BlueWorkType::ZERO,
            1,
            Hash::default(),
        );
        let block1 = Block::new(header1, vec![cb_tx]);

        let commitment1 = processor.process_block(&mut writer, &block1).unwrap();
        assert_ne!(commitment1, Hash::default());
        assert_eq!(processor.utxo_set_len(), 1);

        // 2. Process block 2 spending the matured coinbase at DAA score 20 (20 >= 1 + 10)
        let spend_outpoint = TransactionOutpoint::new(tx1_id, 0);
        let spend_input = TransactionInput::new(spend_outpoint, vec![], 0, 1);
        let spend_output = TransactionOutput::new(950_000, ScriptPublicKey::default());
        let spend_tx = Transaction::new(
            1,
            vec![spend_input],
            vec![spend_output],
            0,
            SubnetworkId::default(),
            0,
            vec![],
        );

        let cb2 = Transaction::new(
            1,
            vec![],
            vec![TransactionOutput::new(500_000, ScriptPublicKey::default())],
            0,
            SubnetworkId::COINBASE,
            0,
            vec![],
        );

        let header2 = Header::new_finalized(
            1,
            vec![vec![block1.header.hash]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            2000,
            0x1e7fffff,
            0,
            20,
            BlueWorkType::ZERO,
            2,
            Hash::default(),
        );
        let block2 = Block::new(header2, vec![cb2, spend_tx]);

        let commitment2 = processor.process_block(&mut writer, &block2).unwrap();
        assert_ne!(commitment2, commitment1);
        // After spending 1 outpoint and creating 2 new outpoints (cb2 + spend_tx output), len is 1 - 1 + 2 = 2
        assert_eq!(processor.utxo_set_len(), 2);
    }
}
