//! Global environment detection (threads, storage locations, core limits).

use once_cell::sync::Lazy;
use std::{env, path::PathBuf};

/// Global environment detection descriptor for the Jio node and pad runtime.
#[derive(Clone, Debug)]
pub struct JiopadEnv {
    /// Number of logical CPU cores available.
    pub logical_cores: usize,
    /// Number of physical CPU cores.
    pub physical_cores: usize,
    /// Configured worker thread count.
    pub worker_threads: usize,
    /// Optional core affinity / limit.
    pub core_limit: Option<usize>,
    /// Base application data directory.
    pub data_dir: PathBuf,
    /// Database storage directory.
    pub db_dir: PathBuf,
    /// Logging directory.
    pub logs_dir: PathBuf,
}

static GLOBAL_ENV: Lazy<JiopadEnv> = Lazy::new(JiopadEnv::detect);

impl JiopadEnv {
    /// Get the globally detected environment.
    pub fn current() -> &'static JiopadEnv {
        &GLOBAL_ENV
    }

    /// Detect current hardware environment, operating system paths, and configuration overrides.
    pub fn detect() -> Self {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let mut sys = sysinfo::System::new();
        sys.refresh_cpu();
        let physical_cores = sys.physical_core_count().unwrap_or(logical_cores);

        // Core limits override
        let core_limit = env::var("JIO_CORE_LIMIT")
            .ok()
            .and_then(|s| s.parse::<usize>().ok());

        // Worker thread count override
        let worker_threads = env::var("JIO_MAX_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| {
                if let Some(limit) = core_limit {
                    limit.min(logical_cores)
                } else {
                    logical_cores
                }
            });

        // Determine default data directory
        let data_dir = env::var("JIO_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_data_dir());

        let db_dir = data_dir.join("datadir2");
        let logs_dir = data_dir.join("logs");

        Self {
            logical_cores,
            physical_cores,
            worker_threads,
            core_limit,
            data_dir,
            db_dir,
            logs_dir,
        }
    }

    /// Ensure that application directories exist on the filesystem.
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.db_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        Ok(())
    }
}

fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("Jiopad")
        } else if let Ok(app_data) = env::var("APPDATA") {
            PathBuf::from(app_data).join("Jiopad")
        } else {
            PathBuf::from("./jiopad_data")
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Jiopad")
        } else {
            PathBuf::from("./jiopad_data")
        }
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".jiopad")
        } else {
            PathBuf::from("./jiopad_data")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        let env = JiopadEnv::detect();
        assert!(env.logical_cores > 0);
        assert!(env.worker_threads > 0);
        assert!(!env.data_dir.as_os_str().is_empty());
    }
}