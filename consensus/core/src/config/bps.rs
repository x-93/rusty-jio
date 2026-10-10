pub const BPS_1: u64 = 1;
pub const BPS_10: u64 = 10;
pub const BPS_20: u64 = 20;
pub const BPS_32: u64 = 32;

pub fn bps_to_target_interval(bps: u64) -> u64 {
    1000 / bps.max(1)
}
