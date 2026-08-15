use crate::model::stores::ghostdag::GhostdagStore;
use crate::model::stores::headers::HeaderStore;
use crate::model::stores::relations::RelationsStore;
use crate::model::stores::statuses::StatusesStore;
use crate::pipeline::header_processor::post_pow_validation::validate_post_pow;
use crate::pipeline::header_processor::pre_ghostdag_validation::validate_pre_ghostdag;
use crate::pipeline::header_processor::pre_pow_validation::validate_pre_pow;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::ghostdag::GhostdagManager;
use crate::processes::past_median_time::PastMedianTimeManager;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_consensus_core::errors::consensus::ConsensusResult;
use jio_consensus_core::hashing::header::header_hash;
use jio_consensus_core::header::Header;
use std::sync::Arc;

pub struct HeaderProcessor {
    header_store: HeaderStore,
    ghostdag_store: GhostdagStore,
    relations_store: RelationsStore,
    statuses_store: StatusesStore,
    ghostdag_manager: GhostdagManager,
    difficulty_manager: DifficultyManager,
    pmt_manager: PastMedianTimeManager,
}

impl HeaderProcessor {
    pub fn new(
        header_store: HeaderStore,
        ghostdag_store: GhostdagStore,
        relations_store: RelationsStore,
        statuses_store: StatusesStore,
        ghostdag_manager: GhostdagManager,
        difficulty_manager: DifficultyManager,
        pmt_manager: PastMedianTimeManager,
    ) -> Self {
        Self {
            header_store,
            ghostdag_store,
            relations_store,
            statuses_store,
            ghostdag_manager,
            difficulty_manager,
            pmt_manager,
        }
    }

    pub fn process_header(&self, header: &Header) -> ConsensusResult<BlockHash> {
        let hash = header_hash(header);

        if self.header_store.has(&hash) {
            return Ok(hash);
        }

        // 1. Pre-PoW validation
        let now = jio_core::time::unix_now() + 60_000; // allow 60 sec future clock drift
        validate_pre_pow(header, now)?;

        // 2. Pre-GhostDAG validation
        let parents = header.direct_parents();
        if !parents.is_empty() {
            let selected_parent = parents[0];
            let pmt = self.pmt_manager.calc_past_median_time(&selected_parent);
            let expected_bits = self.difficulty_manager.calc_target_bits(&selected_parent);
            validate_pre_ghostdag(header, pmt, expected_bits)?;
        }

        // 3. Post-PoW validation
        validate_post_pow(header)?;

        // 4. GhostDAG processing
        let ghostdag_data = self.ghostdag_manager.ghostdag(parents);

        // 5. Store data
        self.header_store.insert(hash, Arc::new(header.clone()));
        self.ghostdag_store.insert(hash, Arc::new(ghostdag_data));
        self.relations_store.insert(hash, parents.to_vec());
        self.statuses_store.set(hash, BlockStatus::StatusHeaderOnly);

        Ok(hash)
    }
}
