use crate::model::stores::ghostdag::GhostdagStoreReader;
use crate::model::stores::headers::HeaderStoreReader;
use crate::processes::difficulty::DifficultyManager;
use crate::processes::past_median_time::{calc_past_median_time, validate_past_median_time};
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::header::Header;
use jio_hashes::Hash;

/// Validates contextual header rules before GHOSTDAG execution:
/// 1. All direct parents must already exist in the HeaderStore.
/// 2. Header timestamp must be strictly greater than Past Median Time (PMT).
/// 3. Header bits must match the target difficulty computed by the DAA engine.
pub fn validate_pre_ghostdag(
    header: &Header,
    header_store: &impl HeaderStoreReader,
    ghostdag_store: &impl GhostdagStoreReader,
    difficulty_manager: &DifficultyManager,
    selected_parent: Hash,
    default_bits: u32,
    pmt_window_size: usize,
) -> BlockProcessResult<()> {
    // 1. Direct parents existence check
    for &parent in header.direct_parents() {
        if !header_store.has(parent) {
            return Err(RuleError::MissingParent(parent));
        }
    }

    // 2. Past Median Time (PMT) check
    if selected_parent != Hash::default() {
        let pmt = calc_past_median_time(
            ghostdag_store,
            header_store,
            selected_parent,
            pmt_window_size,
        );
        validate_past_median_time(header.timestamp, pmt)?;
    }

    // 3. Difficulty Target (DAA) check
    let expected_bits = difficulty_manager.calc_next_required_difficulty(
        ghostdag_store,
        header_store,
        selected_parent,
        default_bits,
    );

    if header.bits != expected_bits {
        log::debug!(
            "Header bits mismatch for block {}: got 0x{:08x}, expected 0x{:08x}",
            header.hash,
            header.bits,
            expected_bits
        );
        return Err(RuleError::InvalidPoW);
    }

    Ok(())
}
