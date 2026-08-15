use crate::model::stores::headers::HeaderStore;
use jio_consensus_core::blockhash::BlockHash;

#[derive(Clone)]
pub struct PastMedianTimeManager {
    header_store: HeaderStore,
    window_size: usize,
}

impl PastMedianTimeManager {
    pub fn new(header_store: HeaderStore, window_size: usize) -> Self {
        Self {
            header_store,
            window_size,
        }
    }

    pub fn calc_past_median_time(&self, selected_parent: &BlockHash) -> u64 {
        let mut timestamps = Vec::with_capacity(self.window_size);
        let mut current = Some(*selected_parent);

        while let Some(hash) = current {
            if let Some(header) = self.header_store.get_header(&hash) {
                timestamps.push(header.timestamp);
                if timestamps.len() >= self.window_size {
                    break;
                }
                current = header.direct_parents().first().copied();
            } else {
                break;
            }
        }

        if timestamps.is_empty() {
            return 0;
        }

        timestamps.sort_unstable();
        timestamps[timestamps.len() / 2]
    }
}
