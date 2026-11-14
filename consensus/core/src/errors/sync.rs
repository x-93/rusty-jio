use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    #[error("Missing block bodies during IBD")]
    MissingBodies,
}
