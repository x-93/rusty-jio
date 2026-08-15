pub use jio_consensus_core::errors::*;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum PipelineError {
    #[error("process error: {0}")]
    Process(String),
    #[error("missing dependency: {0}")]
    MissingDependency(String),
    #[error("pipeline queue is full")]
    QueueFull,
}
