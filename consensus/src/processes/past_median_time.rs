use crate::model::stores::ghostdag::GhostdagStoreReader;
use crate::model::stores::headers::HeaderStoreReader;
use consensus_core::errors::block::RuleError;
use jio_hashes::Hash;

pub const DEFAULT_MEDIAN_TIME_WINDOW_SIZE: usize = 263;

/// Calculates the past median time (PMT) by sampling timestamps of blue blocks
/// along the selected parent chain.
pub fn calc_past_median_time(
    ghostdag_store: &impl GhostdagStoreReader,
    headers_store: &impl HeaderStoreReader,
    selected_parent: Hash,
    window_size: usize,
) -> u64 {
    if selected_parent == Hash::default() {
        return 0;
    }

    let mut timestamps = Vec::with_capacity(window_size);
    let mut current = selected_parent;

    while current != Hash::default() && timestamps.len() < window_size {
        if let Ok(ts) = headers_store.get_timestamp(current) {
            timestamps.push(ts);
        } else {
            break;
        }

        current = ghostdag_store
            .get_selected_parent(current)
            .unwrap_or_default();
    }

    if timestamps.is_empty() {
        return 0;
    }

    timestamps.sort_unstable();
    timestamps[timestamps.len() / 2]
}

/// Validates that a block timestamp is strictly greater than the past median time (PMT).
pub fn validate_past_median_time(
    block_timestamp: u64,
    past_median_time: u64,
) -> Result<(), RuleError> {
    if block_timestamp <= past_median_time {
        Err(RuleError::TimeTooOld(block_timestamp, past_median_time))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::ghostdag::{DbGhostdagStore, GhostdagData, GhostdagStore};
    use crate::model::stores::headers::{DbHeaderStore, HeaderStore};
    use consensus_core::header::Header;
    use consensus_core::{BlockHashMap, BlueWorkType, HashMapCustomHasher};
    use jio_database::prelude::{BatchDbWriter, DB};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_past_median_time() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(DB::open_default(temp_dir.path()).unwrap());
        let mut writer = BatchDbWriter::new(&db);

        let ghostdag_store = DbGhostdagStore::new(db.clone(), 100);
        let headers_store = DbHeaderStore::new(db.clone(), 100);

        // Chain of 5 blocks with timestamps: 100, 110, 105, 120, 130
        let timestamps = [100u64, 110, 105, 120, 130];
        let mut prev_hash = Hash::default();

        for (i, &ts) in timestamps.iter().enumerate() {
            let hash = Hash::from_le_u64([(i + 1) as u64, 0, 0, 0]);
            let header = Arc::new(Header::new_finalized(
                1,
                vec![],
                Hash::default(),
                Hash::default(),
                Hash::default(),
                ts,
                0x1e7fffff,
                0,
                i as u64,
                BlueWorkType::ZERO,
                i as u64,
                Hash::default(),
            ));
            headers_store.insert(&mut writer, hash, header).unwrap();

            let gd = Arc::new(GhostdagData::new(
                i as u64,
                BlueWorkType::ZERO,
                prev_hash,
                vec![],
                vec![],
                BlockHashMap::new(),
            ));
            ghostdag_store.insert(&mut writer, hash, gd).unwrap();
            prev_hash = hash;
        }

        // Sorted timestamps of past 5 blocks: [100, 105, 110, 120, 130] -> median is 110 (index 5/2 = 2)
        let pmt = calc_past_median_time(&ghostdag_store, &headers_store, prev_hash, 10);
        assert_eq!(pmt, 110);

        // Validation checks
        assert!(validate_past_median_time(111, pmt).is_ok());
        assert!(validate_past_median_time(110, pmt).is_err());
        assert!(validate_past_median_time(100, pmt).is_err());
    }
}
