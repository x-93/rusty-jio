use crate::log::consts::{DEFAULT_LOG_LEVEL, LOG_FILTER_ENV};
use env_logger::{Builder, Env};
use log::LevelFilter;
use std::io::Write;

pub fn init_logger(default_level: Option<&str>) {
    let filter = default_level.unwrap_or(DEFAULT_LOG_LEVEL);
    let env = Env::default().filter_or(LOG_FILTER_ENV, filter);

    let _ = Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{:<5}] [{}] {}",
                chrono_or_local_time(),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .try_init();
}

pub fn init_logger_with_level(level: LevelFilter) {
    let _ = Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{:<5}] [{}] {}",
                chrono_or_local_time(),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .try_init();
}

fn chrono_or_local_time() -> String {
    let now = crate::time::unix_now();
    format!("{}", now)
}
