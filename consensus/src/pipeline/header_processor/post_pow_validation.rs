use jio_consensus_core::errors::block::BlockRuleError;
use jio_consensus_core::hashing::header::pre_pow_hash;
use jio_consensus_core::header::Header;
use jio_hashes::pow_hash;

pub fn validate_post_pow(header: &Header) -> Result<(), BlockRuleError> {
    let pre_pow = pre_pow_hash(header);
    let p_hash = pow_hash(pre_pow, header.timestamp, header.nonce);

    // PoW target check
    let target_leading_zeros = (32 - (header.bits >> 24)) as usize;
    let actual_leading_zeros = p_hash.as_bytes().iter().take_while(|&&b| b == 0).count();

    if actual_leading_zeros < target_leading_zeros && header.bits != 0x207f_ffff {
        return Err(BlockRuleError::InvalidTx("insufficient proof of work".to_string()));
    }

    Ok(())
}
