//! # Jio Core
//!
//! Foundational async application framework, service supervisor (`Core`),
//! signal handlers, logging, and Tokio runtime configuration for the Jio node.

pub mod assert;
pub mod console;
pub mod core;
pub mod jiopad_env;
pub mod log;
pub mod panic;
pub mod service;
pub mod signals;
pub mod task;
pub mod time;

pub use core::Core;
pub use service::{AsyncService, ServiceError};
pub use signals::Signals;
