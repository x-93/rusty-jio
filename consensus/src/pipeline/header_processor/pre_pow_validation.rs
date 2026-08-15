use jio_consensus_core::constants::BLOCK_VERSION;
use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::header::Header;
use std::collections::HashSet;

const MAX_PARENTS_PER_LEVEL: usize = 32;

pub fn validate_pre_pow(header: &Header, max_future_timestamp: u64) -> Result<(), BlockRuleError> {
    if header.version != BLOCK_VERSION {
        return Err(BlockRuleError::BadVersion(header.version));
    }

    if header.timestamp > max_future_timestamp {
        return Err(BlockRuleError::TimeTooNew(header.timestamp));
    }

    let parents = header.direct_parents();
    if !parents.is_empty() {
        if parents.len() > MAX_PARENTS_PER_LEVEL {
            return Err(BlockRuleError::TooManyParents(
                parents.len(),
                MAX_PARENTS_PER_LEVEL,
            ));
        }

        let mut seen = HashSet::with_capacity(parents.len());
        for parent in parents {
            if !seen.insert(*parent) {
                return Err(BlockRuleError::DuplicateParent);
            }
        }
    }

    Ok(())
}
