//! Cache policies and LRU eviction tracking.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachePolicy {
    Empty,
    Count(usize),
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self::Count(10_000)
    }
}
