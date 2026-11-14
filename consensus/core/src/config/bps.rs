/// Blocks per second definitions and scaling factors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bps(pub u64);

impl Bps {
    pub const BPS_1: Self = Self(1);
    pub const BPS_10: Self = Self(10);

    pub fn target_time_ms(&self) -> u64 {
        1000 / self.0.max(1)
    }
}
