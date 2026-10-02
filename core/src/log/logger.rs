//! Structured logger initialization.

pub fn init_logger(default_level: Option<&str>) {
    let level = default_level.unwrap_or(super::consts::DEFAULT_LOG_LEVEL);
    println!("[LOGGER] Initialized logging at level: {}", level);
}
