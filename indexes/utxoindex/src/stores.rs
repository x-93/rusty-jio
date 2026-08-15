use crate::model::CirculatingSupply;
use jio_indexes_core::IndexedUtxos;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct UtxoIndexStores {
    pub indexed_utxos: Arc<RwLock<IndexedUtxos>>,
    pub circulating_supply: Arc<RwLock<CirculatingSupply>>,
}

impl UtxoIndexStores {
    pub fn new() -> Self {
        Self {
            indexed_utxos: Arc::new(RwLock::new(IndexedUtxos::new())),
            circulating_supply: Arc::new(RwLock::new(CirculatingSupply::default())),
        }
    }
}
