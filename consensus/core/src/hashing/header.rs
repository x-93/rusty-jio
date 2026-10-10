use crate::header::Header;
use jio_hashes::{BlockHash, Hash, Hasher, ProofOfWorkHash};

pub fn hash(header: &Header) -> Hash {
    let mut hasher = BlockHash::new();
    serialize_header(&mut hasher, header);
    hasher.update(header.nonce.to_le_bytes());
    hasher.finalize()
}

pub fn pre_pow_hash(header: &Header) -> Hash {
    let mut hasher = ProofOfWorkHash::new();
    serialize_header(&mut hasher, header);
    hasher.finalize()
}

fn serialize_header<H: Hasher>(hasher: &mut H, header: &Header) {
    hasher.update(header.version.to_le_bytes());
    hasher.update((header.parents_by_level.len() as u64).to_le_bytes());
    for level in &header.parents_by_level {
        hasher.update((level.len() as u64).to_le_bytes());
        for parent in level {
            hasher.update(parent.as_bytes());
        }
    }
    hasher.update(header.hash_merkle_root.as_bytes());
    hasher.update(header.accepted_id_merkle_root.as_bytes());
    hasher.update(header.utxo_commitment.as_bytes());
    hasher.update(header.timestamp.to_le_bytes());
    hasher.update(header.bits.to_le_bytes());
    hasher.update(header.daa_score.to_le_bytes());
    hasher.update(header.blue_work.to_le_bytes());
    hasher.update(header.blue_score.to_le_bytes());
    hasher.update(header.pruning_point.as_bytes());
}