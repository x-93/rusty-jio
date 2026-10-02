//! Environment variables and default path configuration.

use std::path::PathBuf;

pub const DEFAULT_DATA_DIR: &str = ".jiopad";

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("JIOPAD_DATA_DIR") {
        PathBuf::from(dir)
    } else if let Some(home) = dirs_home() {
        home.join(DEFAULT_DATA_DIR)
    } else {
        PathBuf::from(DEFAULT_DATA_DIR)
    }
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}
