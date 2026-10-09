//! Jio core runtime, service lifecycle, task supervisor, and environment detection.

pub mod jiopad_env;
pub mod task;

pub use jiopad_env::JiopadEnv;
pub use task::{cooperative_tick, AsyncRuntime, AsyncService, ServiceSupervisor, ShutdownSignal, TickService};
