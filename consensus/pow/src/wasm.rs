use crate::matrix::Matrix;
use jio_hashes::Hash;

#[derive(Clone, Debug)]
pub struct PowState {
    pub matrix: Matrix,
    pub pre_pow_hash: Hash,
    pub target: jio_math::Uint256,
}

impl PowState {
    pub fn new(pre_pow_hash: Hash, target: jio_math::Uint256) -> Self {
        let matrix = Matrix::generate(&pre_pow_hash);
        Self {
            matrix,
            pre_pow_hash,
            target,
        }
    }

    pub fn check_pow(&self, nonce: u64) -> bool {
        let mut hasher = jio_hashes::ProofOfWorkHash::new();
        hasher.write(self.pre_pow_hash);
        hasher.write(&nonce.to_le_bytes());
        let intermediate_hash = hasher.finalize();

        let pow_hash = self.matrix.heavy_hash(&intermediate_hash);
        let hash_val = jio_math::Uint256::from_le_bytes(pow_hash.as_bytes());
        hash_val <= self.target
    }
}
