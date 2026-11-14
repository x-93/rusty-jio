use crate::model::stores::daa::DbDaaStore;
use crate::model::stores::depth::DbDepthStore;
use crate::model::stores::ghostdag::DbGhostdagStore;
use crate::model::stores::headers::DbHeaderStore;
use crate::model::stores::reachability::DbReachabilityStore;
use crate::model::stores::relations::DbRelationsStore;
use crate::model::stores::statuses::DbStatusesStore;
use crate::model::stores::utxo_diffs::DbUtxoDiffsStore;
use jio_database::prelude::DB;
use parking_lot::RwLock;
use std::sync::Arc;

pub const DEFAULT_CACHE_SIZE: u64 = 10_000;

#[derive(Clone)]
pub struct ConsensusStorage {
    pub db: Arc<DB>,
    pub headers: Arc<DbHeaderStore>,
    pub relations: Arc<DbRelationsStore>,
    pub ghostdag: Arc<RwLock<DbGhostdagStore>>,
    pub reachability: Arc<RwLock<DbReachabilityStore>>,
    pub statuses: Arc<RwLock<DbStatusesStore>>,
    pub depth: Arc<DbDepthStore>,
    pub daa: Arc<DbDaaStore>,
    pub utxo_diffs: Arc<DbUtxoDiffsStore>,
}

impl ConsensusStorage {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            db: db.clone(),
            headers: Arc::new(DbHeaderStore::new(db.clone(), cache_size)),
            relations: Arc::new(DbRelationsStore::new(db.clone(), cache_size)),
            ghostdag: Arc::new(RwLock::new(DbGhostdagStore::new(db.clone(), cache_size))),
            reachability: Arc::new(RwLock::new(DbReachabilityStore::new(
                db.clone(),
                cache_size,
            ))),
            statuses: Arc::new(RwLock::new(DbStatusesStore::new(db.clone(), cache_size))),
            depth: Arc::new(DbDepthStore::new(db.clone(), cache_size)),
            daa: Arc::new(DbDaaStore::new(db.clone(), cache_size)),
            utxo_diffs: Arc::new(DbUtxoDiffsStore::new(db, cache_size)),
        }
    }
}
