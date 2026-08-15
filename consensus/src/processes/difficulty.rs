use crate::model::stores::headers::HeaderStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::errors::difficulty::DifficultyError;
use jio_hashes::Hash;
use jio_math::Uint256;

/// Converts a 32-bit compact representation (bits) into a 256-bit target integer.
pub fn compact_to_target(bits: u32) -> Result<Uint256, DifficultyError> {
    let exponent = (bits >> 24) as usize;
    let is_negative = (bits & 0x0080_0000) != 0;
    let mantissa = bits & 0x007f_ffff;

    if is_negative {
        return Err(DifficultyError::TargetOutOfRange(bits));
    }

    if mantissa == 0 || exponent == 0 {
        return Ok(Uint256::ZERO);
    }

    if exponent > 32 {
        return Err(DifficultyError::TargetOutOfRange(bits));
    }

    let mut target = Uint256::from_u64(mantissa as u64);
    if exponent <= 3 {
        target = target >> (8 * (3 - exponent));
    } else {
        target = target << (8 * (exponent - 3));
    }

    Ok(target)
}

/// Converts a 256-bit target integer into a 32-bit compact representation (bits).
pub fn target_to_compact(target: Uint256) -> u32 {
    if target.is_zero() {
        return 0;
    }

    let num_bits = target.bits();
    let mut num_bytes = (num_bits + 7) / 8;

    let mut mantissa = if num_bytes <= 3 {
        (target.low_u64() << (8 * (3 - num_bytes))) as u32
    } else {
        (target >> (8 * (num_bytes - 3))).low_u64() as u32
    };

    if (mantissa & 0x0080_0000) != 0 {
        mantissa >>= 8;
        num_bytes += 1;
    }

    ((num_bytes as u32) << 24) | (mantissa & 0x007f_ffff)
}

/// Computes the blue work unit from target difficulty bits.
pub fn calc_work_from_bits(bits: u32) -> Result<Uint256, DifficultyError> {
    let target = compact_to_target(bits)?;
    if target.is_zero() {
        return Ok(Uint256::ZERO);
    }
    // work = (~target) / (target + 1) + 1
    let (work, _) = (!target).div_rem(target + Uint256::ONE);
    Ok(work + Uint256::ONE)
}

/// Checks if a given hash satisfies the compact difficulty target.
pub fn check_hash_meets_difficulty(hash: &Hash, bits: u32) -> Result<bool, DifficultyError> {
    let target = compact_to_target(bits)?;
    let hash_uint = Uint256::from_le_bytes(hash.as_bytes());
    Ok(hash_uint <= target)
}

#[derive(Clone)]
pub struct DifficultyManager {
    header_store: HeaderStore,
    _target_time_per_block: u64,
    default_bits: u32,
}

impl DifficultyManager {
    pub fn new(header_store: HeaderStore, target_time_per_block: u64, default_bits: u32) -> Self {
        Self {
            header_store,
            _target_time_per_block: target_time_per_block,
            default_bits,
        }
    }

    pub fn calc_daa_score(&self, selected_parent: &BlockHash) -> u64 {
        self.header_store
            .get_daa_score(selected_parent)
            .map(|score| score + 1)
            .unwrap_or(0)
    }

    pub fn calc_target_bits(&self, selected_parent: &BlockHash) -> u32 {
        self.header_store
            .get_bits(selected_parent)
            .unwrap_or(self.default_bits)
    }

    pub fn check_meets_difficulty(&self, hash: &Hash, bits: u32) -> bool {
        check_hash_meets_difficulty(hash, bits).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_target_roundtrip() {
        let bits_list = [0x1e7f_ffff, 0x207f_ffff, 0x1d00ffff, 0x1b0404cb];

        for &bits in &bits_list {
            let target = compact_to_target(bits).expect("valid bits");
            let compact = target_to_compact(target);
            assert_eq!(bits, compact, "failed for bits: {:#x}", bits);
        }
    }

    #[test]
    fn test_zero_target() {
        let target = compact_to_target(0).unwrap();
        assert_eq!(target, Uint256::ZERO);
        assert_eq!(target_to_compact(Uint256::ZERO), 0);
    }

    #[test]
    fn test_negative_bits_rejected() {
        let invalid_negative_bits = 0x1e80_0001;
        assert!(compact_to_target(invalid_negative_bits).is_err());
    }

    #[test]
    fn test_check_hash_meets_difficulty() {
        let bits = 0x207f_ffff;
        let mut low_hash = [0u8; 32];
        low_hash[0] = 0x01;
        let meets = check_hash_meets_difficulty(&Hash::from_bytes(low_hash), bits).unwrap();
        assert!(meets);

        let high_hash = [0xffu8; 32];
        let fails = check_hash_meets_difficulty(&Hash::from_bytes(high_hash), bits).unwrap();
        assert!(!fails);
    }
}
