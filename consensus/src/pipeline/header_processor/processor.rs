use super::{
    post_pow_validation::validate_post_pow, pre_ghostdag_validation::validate_pre_ghostdag,
    pre_pow_validation::validate_pre_pow,
};
use crate::model::stores::ghostdag::{GhostdagData, GhostdagStore};
use crate::model::stores::headers::HeaderStore;
use crate::model::stores::reachability::ReachabilityStore;
use crate::model::stores::relations::RelationsStore;
use crate::model::stores::statuses::StatusesStore;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::ghostdag::GhostdagManager;
use crate::processes::ghostdag::protocol::find_selected_parent;
use crate::processes::reachability::ReachabilityManager;
use consensus_core::blockstatus::BlockStatus;
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::header::Header;
use jio_database::prelude::DbWriter;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct HeaderProcessor<
    THeaderStore: HeaderStore + Send + Sync,
    TRelationsStore: RelationsStore + Send + Sync,
    TReachabilityStore: ReachabilityStore + Send + Sync,
    TGhostdagStore: GhostdagStore + Send + Sync,
    TStatusesStore: StatusesStore + Send + Sync,
> {
    header_store: Arc<THeaderStore>,
    relations_store: Arc<TRelationsStore>,
    reachability_manager: Arc<ReachabilityManager<TReachabilityStore, TRelationsStore>>,
    ghostdag_manager: Arc<
        GhostdagManager<
            TGhostdagStore,
            TRelationsStore,
            THeaderStore,
            ReachabilityManager<TReachabilityStore, TRelationsStore>,
        >,
    >,
    ghostdag_store: Arc<RwLock<TGhostdagStore>>,
    statuses_store: Arc<RwLock<TStatusesStore>>,
    difficulty_manager: Arc<DifficultyManager>,
    timestamp_deviation_tolerance_ms: u64,
    default_bits: u32,
    genesis_hash: Hash,
}

impl<
    THeaderStore: HeaderStore + Send + Sync,
    TRelationsStore: RelationsStore + Send + Sync,
    TReachabilityStore: ReachabilityStore + Send + Sync,
    TGhostdagStore: GhostdagStore + Send + Sync,
    TStatusesStore: StatusesStore + Send + Sync,
>
    HeaderProcessor<
        THeaderStore,
        TRelationsStore,
        TReachabilityStore,
        TGhostdagStore,
        TStatusesStore,
    >
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        header_store: Arc<THeaderStore>,
        relations_store: Arc<TRelationsStore>,
        reachability_manager: Arc<ReachabilityManager<TReachabilityStore, TRelationsStore>>,
        ghostdag_manager: Arc<
            GhostdagManager<
                TGhostdagStore,
                TRelationsStore,
                THeaderStore,
                ReachabilityManager<TReachabilityStore, TRelationsStore>,
            >,
        >,
        ghostdag_store: Arc<RwLock<TGhostdagStore>>,
        statuses_store: Arc<RwLock<TStatusesStore>>,
        difficulty_manager: Arc<DifficultyManager>,
        timestamp_deviation_tolerance_ms: u64,
        default_bits: u32,
        genesis_hash: Hash,
    ) -> Self {
        Self {
            header_store,
            relations_store,
            reachability_manager,
            ghostdag_manager,
            ghostdag_store,
            statuses_store,
            difficulty_manager,
            timestamp_deviation_tolerance_ms,
            default_bits,
            genesis_hash,
        }
    }

    /// Fully processes, validates, and commits an incoming block header into the consensus DAG.
    pub fn process_header(
        &self,
        writer: &mut impl DbWriter,
        header: Arc<Header>,
    ) -> BlockProcessResult<Arc<GhostdagData>> {
        let block_hash = header.hash;

        // 1. Duplicate check
        if self.header_store.has(block_hash) {
            return Err(RuleError::BlockAlreadyExists(block_hash));
        }

        let is_genesis = block_hash == self.genesis_hash;

        // 2. Pre-PoW validation (timestamp tolerance, parent count)
        let now_ms = jio_core::time::unix_now_ms();
        validate_pre_pow(
            &header,
            now_ms,
            self.timestamp_deviation_tolerance_ms,
            is_genesis,
        )?;

        // 3. Post-PoW validation (meets target bits)
        validate_post_pow(&header)?;

        let parents = header.direct_parents();

        // 4. Pre-GHOSTDAG validation (parents existence, PMT, DAA bits)
        let selected_parent = if is_genesis {
            Hash::default()
        } else {
            let gd_store = self.ghostdag_store.read();
            find_selected_parent(&*gd_store, parents).ok_or(RuleError::NoParents)?
        };

        if !is_genesis {
            let gd_store = self.ghostdag_store.read();
            validate_pre_ghostdag(
                &header,
                &*self.header_store,
                &*gd_store,
                &self.difficulty_manager,
                selected_parent,
                self.default_bits,
                263,
            )?;
        }

        // 5. Commit relations and DAG topology
        self.header_store
            .insert(writer, block_hash, header.clone())
            .map_err(|_| RuleError::KnownInvalid(block_hash))?;

        let parents_vec = Arc::new(parents.to_vec());
        self.relations_store
            .insert(writer, block_hash, parents_vec.clone())
            .map_err(|_| RuleError::KnownInvalid(block_hash))?;

        for &parent in parents {
            self.relations_store
                .add_child(writer, parent, block_hash)
                .map_err(|_| RuleError::KnownInvalid(block_hash))?;
        }

        // 6. Reachability tree update
        if is_genesis {
            self.reachability_manager
                .init(writer, block_hash)
                .map_err(|_| RuleError::KnownInvalid(block_hash))?;
        } else {
            self.reachability_manager
                .add_block(writer, block_hash, selected_parent)
                .map_err(|_| RuleError::KnownInvalid(block_hash))?;
        }

        // 7. GHOSTDAG consensus election
        let ghostdag_data = self
            .ghostdag_manager
            .add_block(writer, block_hash, parents_vec)
            .map_err(|_| RuleError::KnownInvalid(block_hash))?;

        // 8. Update block status
        self.statuses_store
            .write()
            .set(writer, block_hash, BlockStatus::STATUS_HEADER_ONLY)
            .map_err(|_| RuleError::KnownInvalid(block_hash))?;

        Ok(ghostdag_data)
    }
}
