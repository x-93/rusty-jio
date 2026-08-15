pub struct CachePolicy {
    pub max_headers: usize,
    pub max_blocks: usize,
    pub max_utxos: usize,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            max_headers: 100_000,
            max_blocks: 10_000,
            max_utxos: 1_000_000,
        }
    }
}
