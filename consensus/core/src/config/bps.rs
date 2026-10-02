//! Block Per Second (BPS) parameters.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bps {
    Bps1,
    Bps10,
}

impl Bps {
    pub fn target_time_per_block_ms(&self) -> u64 {
        match self {
            Self::Bps1 => 1000,
            Self::Bps10 => 100,
        }
    }

    pub fn ghostdag_k(&self) -> u64 {
        match self {
            Self::Bps1 => 18,
            Self::Bps10 => 32,
        }
    }
}
