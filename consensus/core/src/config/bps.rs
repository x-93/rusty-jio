/// Calculates the k parameter of the GHOSTDAG protocol such that anticones larger than k will be created
/// with probability less than `delta` (follows eq. 1 from section 4.2 of the PHANTOM paper)
/// `x` is expected to be 2Dλ where D is the maximal network delay and λ is the block mining rate.
/// `delta` is an upper bound for the probability of anticones larger than k.
/// Returns the minimal k such that the above conditions hold.
pub fn calculate_ghostdag_k(x: f64, delta: f64) -> u64 {
    assert!(x > 0.0);
    assert!(delta > 0.0 && delta < 1.0);
    let (mut k_hat, mut sigma, mut fraction, exp) = (0u64, 0.0, 1.0, std::f64::consts::E.powf(-x));
    loop {
        sigma += exp * fraction;
        if 1.0 - sigma < delta {
            return k_hat;
        }
        k_hat += 1;
        fraction *= x / k_hat as f64 // Computes x^k_hat/k_hat!
    }
}

/// Helper formulas for scaling BlockDAG parameters according to Blocks Per Second (BPS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bps {
    Bps1 = 1,
    Bps10 = 10,
}

impl Bps {
    pub const fn bps(&self) -> u64 {
        match self {
            Bps::Bps1 => 1,
            Bps::Bps10 => 10,
        }
    }

    /// Computes the GHOSTDAG anticone K parameter from BPS.
    /// At 1 BPS, K = 18. At 10 BPS, K = 124.
    pub const fn ghostdag_k(&self) -> u64 {
        match self {
            Bps::Bps1 => 18,
            Bps::Bps10 => 124,
        }
    }

    /// Block interval in milliseconds.
    pub const fn target_time_per_block_ms(&self) -> u64 {
        match self {
            Bps::Bps1 => 1000,
            Bps::Bps10 => 100,
        }
    }

    /// Difficulty adjustment sliding window size.
    pub const fn difficulty_window_size(&self) -> usize {
        match self {
            Bps::Bps1 => 2641,
            Bps::Bps10 => 26410,
        }
    }
}
