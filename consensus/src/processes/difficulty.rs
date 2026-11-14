use crate::model::stores::ghostdag::GhostdagStoreReader;
use crate::model::stores::headers::HeaderStoreReader;
use jio_hashes::Hash;
use jio_math::{Uint256, Uint320};

pub const MIN_DIFFICULTY_WINDOW_LEN: usize = 10;
pub const LEGACY_DIFFICULTY_WINDOW_SIZE: usize = 2641;

#[derive(Clone, Debug)]
pub struct DifficultyManager {
    target_time_per_block: u64,
    min_window_size: usize,
    max_window_size: usize,
    max_difficulty_target: Uint256,
}

impl DifficultyManager {
    pub fn new(
        target_time_per_block: u64,
        min_window_size: usize,
        max_window_size: usize,
        max_difficulty_target: Uint256,
    ) -> Self {
        Self {
            target_time_per_block,
            min_window_size,
            max_window_size,
            max_difficulty_target,
        }
    }

    /// Calculates the required compact target bits for the next block given its selected parent.
    pub fn calc_next_required_difficulty(
        &self,
        ghostdag_store: &impl GhostdagStoreReader,
        headers_store: &impl HeaderStoreReader,
        selected_parent: Hash,
        default_bits: u32,
    ) -> u32 {
        if selected_parent == Hash::default() {
            return default_bits;
        }

        // 1. Collect window headers along the selected parent chain
        let mut window = Vec::with_capacity(self.max_window_size);
        let mut current = selected_parent;

        while current != Hash::default() && window.len() < self.max_window_size {
            if let (Ok(ts), Ok(bits)) = (
                headers_store.get_timestamp(current),
                headers_store.get_bits(current),
            ) {
                window.push((ts, bits));
            } else {
                break;
            }

            current = ghostdag_store
                .get_selected_parent(current)
                .unwrap_or_default();
        }

        // If window is smaller than minimum required, preserve parent's bits
        if window.len() < self.min_window_size {
            return headers_store
                .get_bits(selected_parent)
                .unwrap_or(default_bits);
        }

        let newest_timestamp = window[0].0;
        let oldest_timestamp = window[window.len() - 1].0;

        // 2. Measure actual vs expected duration
        let actual_duration = newest_timestamp.saturating_sub(oldest_timestamp).max(1);
        let expected_duration = ((window.len() - 1) as u64 * self.target_time_per_block).max(1);

        // 3. Compute average target across window
        // Use Uint320 to prevent overflow during sum and multiplication
        let mut sum_target = Uint320::ZERO;
        for &(_, bits) in &window {
            let target256 = Uint256::from_compact_target_bits(bits);
            let target320 = Uint320([
                target256.0[0],
                target256.0[1],
                target256.0[2],
                target256.0[3],
                0,
            ]);
            sum_target = sum_target + target320;
        }
        let avg_target = sum_target / Uint320::from_u64(window.len() as u64);

        // 4. Adjust target: NewTarget = AvgTarget * actual_duration / expected_duration
        let adjusted_target = (avg_target * Uint320::from_u64(actual_duration))
            / Uint320::from_u64(expected_duration);

        // 5. Clamp to max_difficulty_target and ensure non-zero
        let max_target_320 = Uint320([
            self.max_difficulty_target.0[0],
            self.max_difficulty_target.0[1],
            self.max_difficulty_target.0[2],
            self.max_difficulty_target.0[3],
            0,
        ]);

        let final_target256 = if adjusted_target >= max_target_320 {
            self.max_difficulty_target
        } else if adjusted_target.is_zero() {
            Uint256::from_u64(1)
        } else {
            Uint256([
                adjusted_target.0[0],
                adjusted_target.0[1],
                adjusted_target.0[2],
                adjusted_target.0[3],
            ])
        };

        final_target256.compact_target_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::ghostdag::{DbGhostdagStore, GhostdagData, GhostdagStore};
    use crate::model::stores::headers::{DbHeaderStore, HeaderStore};
    use consensus_core::config::constants::consensus::MAX_DIFFICULTY_TARGET;
    use consensus_core::header::Header;
    use consensus_core::{BlockHashMap, BlueWorkType, HashMapCustomHasher};
    use jio_database::prelude::{BatchDbWriter, DB};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_difficulty_adjustment() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(DB::open_default(temp_dir.path()).unwrap());
        let mut writer = BatchDbWriter::new(&db);

        let ghostdag_store = DbGhostdagStore::new(db.clone(), 100);
        let headers_store = DbHeaderStore::new(db.clone(), 100);

        let target_time_ms = 1000u64; // 1 second per block
        let diff_manager = DifficultyManager::new(target_time_ms, 5, 20, MAX_DIFFICULTY_TARGET);

        let initial_bits = 0x1e7fffff;
        let mut prev_hash = Hash::default();

        // Case A: Blocks produced faster than target (every 500ms instead of 1000ms)
        // Difficulty should increase -> Target should decrease
        for i in 0..10 {
            let hash = Hash::from_le_u64([(i + 1) as u64, 0, 0, 0]);
            let timestamp = 10_000 + i as u64 * 500;
            let header = Arc::new(Header::new_finalized(
                1,
                vec![],
                Hash::default(),
                Hash::default(),
                Hash::default(),
                timestamp,
                initial_bits,
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

        let new_bits = diff_manager.calc_next_required_difficulty(
            &ghostdag_store,
            &headers_store,
            prev_hash,
            initial_bits,
        );

        let initial_target = Uint256::from_compact_target_bits(initial_bits);
        let adjusted_target = Uint256::from_compact_target_bits(new_bits);

        // Since blocks were produced at 2x rate, adjusted target must be lower (harder)
        assert!(adjusted_target < initial_target);
    }
}
