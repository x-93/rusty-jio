use crate::model::stores::ghostdag::GhostdagStore;
use crate::model::stores::headers::HeaderStore;
use crate::model::stores::selected_chain::SelectedChainStore;
use crate::model::stores::statuses::StatusesStore;
use crate::model::stores::tips::TipsStore;
use crate::model::stores::utxo_set::UtxoSetStore;
use crate::model::stores::virtual_state::{VirtualState, VirtualStateStore};
use crate::pipeline::virtual_processor::utxo_validation::validate_and_apply_tx_utxo;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::ghostdag::GhostdagManager;
use crate::processes::past_median_time::PastMedianTimeManager;
use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::errors::consensus::{ConsensusError, ConsensusResult};
use jio_consensus_core::hashing::header::header_hash;
use jio_consensus_core::utxo::UtxoDiff;
use jio_hashes::Hash;
use std::sync::Arc;

pub struct VirtualProcessor {
    utxo_set_store: UtxoSetStore,
    virtual_state_store: VirtualStateStore,
    selected_chain_store: SelectedChainStore,
    _ghostdag_store: GhostdagStore,
    _header_store: HeaderStore,
    statuses_store: StatusesStore,
    tips_store: TipsStore,
    ghostdag_manager: GhostdagManager,
    difficulty_manager: DifficultyManager,
    pmt_manager: PastMedianTimeManager,
    coinbase_maturity: u64,
}

impl VirtualProcessor {
    pub fn new(
        utxo_set_store: UtxoSetStore,
        virtual_state_store: VirtualStateStore,
        selected_chain_store: SelectedChainStore,
        ghostdag_store: GhostdagStore,
        header_store: HeaderStore,
        statuses_store: StatusesStore,
        tips_store: TipsStore,
        ghostdag_manager: GhostdagManager,
        difficulty_manager: DifficultyManager,
        pmt_manager: PastMedianTimeManager,
        coinbase_maturity: u64,
    ) -> Self {
        Self {
            utxo_set_store,
            virtual_state_store,
            selected_chain_store,
            _ghostdag_store: ghostdag_store,
            _header_store: header_store,
            statuses_store,
            tips_store,
            ghostdag_manager,
            difficulty_manager,
            pmt_manager,
            coinbase_maturity,
        }
    }

    pub fn process_block(&self, block: &Block) -> ConsensusResult<BlockHash> {
        let hash = header_hash(&block.header);

        // 1. Update DAG tips: remove parents from tips, add new block
        for parent in block.header.direct_parents() {
            self.tips_store.remove(parent);
        }
        self.tips_store.add(hash);

        // 2. Validate transactions and compute UTXO diff
        let mut diff = UtxoDiff::default();
        let daa_score = block.header.daa_score;

        for tx in &block.transactions {
            validate_and_apply_tx_utxo(
                tx,
                &self.utxo_set_store,
                daa_score,
                self.coinbase_maturity,
                &mut diff,
            )?;
        }

        // 3. Commit UTXO diff
        self.utxo_set_store.apply_diff(&diff).map_err(|e| {
            ConsensusError::General(format!("failed to apply UTXO diff: {e}"))
        })?;

        // 4. Update selected chain & virtual state
        let tips = self.tips_store.get_tips();
        let virtual_ghostdag = self.ghostdag_manager.ghostdag(&tips);

        let selected_parent = virtual_ghostdag.selected_parent;
        self.selected_chain_store.set_tip(selected_parent);

        let pmt = if selected_parent != jio_consensus_core::blockhash::ORIGIN {
            self.pmt_manager.calc_past_median_time(&selected_parent)
        } else {
            0
        };

        let bits = if selected_parent != jio_consensus_core::blockhash::ORIGIN {
            self.difficulty_manager.calc_target_bits(&selected_parent)
        } else {
            0x1e7f_ffff
        };

        let virtual_state = VirtualState {
            parents: tips,
            daa_score: virtual_ghostdag.blue_score,
            bits,
            past_median_time: pmt,
            blue_score: virtual_ghostdag.blue_score,
            selected_parent,
            utxo_commitment: Hash::default(),
        };

        self.virtual_state_store.set(Arc::new(virtual_state));
        self.statuses_store.set(hash, BlockStatus::StatusUTXOValid);

        Ok(hash)
    }
}
