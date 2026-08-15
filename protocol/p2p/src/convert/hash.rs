use jio_hashes::Hash;

pub fn hash_to_bytes(hash: &Hash) -> [u8; 32] {
    hash.as_bytes()
}

pub fn bytes_to_hash(bytes: &[u8; 32]) -> Hash {
    Hash::from_bytes(*bytes)
}
