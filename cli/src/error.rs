use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("general error: {0}")]
    General(String),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("wallet error: {0}")]
    Wallet(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}
