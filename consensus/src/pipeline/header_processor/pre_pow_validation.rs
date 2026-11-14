use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::header::Header;

/// Validates header fields that can be verified prior to proof-of-work checking.
pub fn validate_pre_pow(
    header: &Header,
    current_time_ms: u64,
    timestamp_deviation_tolerance_ms: u64,
    is_genesis: bool,
) -> BlockProcessResult<()> {
    // 1. Ensure block timestamp is not too far in the future
    let max_allowed_time = current_time_ms.saturating_add(timestamp_deviation_tolerance_ms);
    if header.timestamp > max_allowed_time {
        return Err(RuleError::TimeTooMuchInTheFuture(header.timestamp));
    }

    // 2. Non-genesis blocks must specify at least one parent
    if !is_genesis && (header.parents_by_level.is_empty() || header.parents_by_level[0].is_empty())
    {
        return Err(RuleError::NoParents);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::BlueWorkType;
    use jio_hashes::Hash;

    #[test]
    fn test_pre_pow_future_timestamp_rejected() {
        let header = Header::new_finalized(
            1,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            20_000,
            0x1e7fffff,
            0,
            0,
            BlueWorkType::ZERO,
            0,
            Hash::default(),
        );

        // Current time 10_000 with 5_000 tolerance -> max allowed is 15_000. Header at 20_000 is rejected!
        assert_eq!(
            validate_pre_pow(&header, 10_000, 5_000, false),
            Err(RuleError::TimeTooMuchInTheFuture(20_000))
        );

        // Within tolerance -> allowed
        assert!(validate_pre_pow(&header, 16_000, 5_000, false).is_ok());
    }

    #[test]
    fn test_pre_pow_no_parents_rejected() {
        let header = Header::new_finalized(
            1,
            vec![],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            10_000,
            0x1e7fffff,
            0,
            0,
            BlueWorkType::ZERO,
            0,
            Hash::default(),
        );

        assert_eq!(
            validate_pre_pow(&header, 10_000, 5_000, false),
            Err(RuleError::NoParents)
        );

        // Genesis is exempt from parent requirement
        assert!(validate_pre_pow(&header, 10_000, 5_000, true).is_ok());
    }
}
