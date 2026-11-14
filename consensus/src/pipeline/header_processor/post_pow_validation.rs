use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::header::Header;
use jio_math::Uint256;

/// Validates that the header satisfies the Proof-of-Work target difficulty encoded in `bits`.
pub fn validate_post_pow(header: &Header) -> BlockProcessResult<()> {
    let pre_pow = header.pre_pow_hash();
    let target = Uint256::from_compact_target_bits(header.bits);
    if jio_pow::check_proof_of_work(pre_pow, header.timestamp, header.nonce, target).is_err() {
        return Err(RuleError::InvalidPoW);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::BlueWorkType;
    use jio_hashes::Hash;

    #[test]
    fn test_post_pow_validation() {
        // High target (easy difficulty) where nonce can easily meet target
        let mut header = Header::new_finalized(
            1,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1000,
            0x207fffff,
            0,
            0,
            BlueWorkType::ZERO,
            0,
            Hash::default(),
        );

        // Find a valid nonce
        let pre_pow = header.pre_pow_hash();
        let target = Uint256::from_compact_target_bits(header.bits);
        let mut valid_nonce = 0;
        for n in 0..10_000 {
            if jio_pow::check_proof_of_work(pre_pow, header.timestamp, n, target).is_ok() {
                valid_nonce = n;
                break;
            }
        }

        header.nonce = valid_nonce;
        header.finalize();
        assert!(validate_post_pow(&header).is_ok());

        // Corrupted bits or impossible target -> rejected
        header.bits = 0x01000001; // extremely difficult
        assert_eq!(validate_post_pow(&header), Err(RuleError::InvalidPoW));
    }
}
