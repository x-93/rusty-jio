use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::header::Header;

pub fn validate_pre_ghostdag(
    header: &Header,
    past_median_time: u64,
    expected_bits: u32,
) -> Result<(), BlockRuleError> {
    if !header.direct_parents().is_empty() && header.timestamp <= past_median_time {
        return Err(BlockRuleError::TimeTooOld(header.timestamp, past_median_time));
    }

    if header.bits != expected_bits {
        return Err(BlockRuleError::InvalidTx(format!(
            "unexpected difficulty bits: expected {expected_bits:#x}, got {:#x}",
            header.bits
        )));
    }

    Ok(())
}
