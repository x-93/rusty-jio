use crate::error::CliError;

pub type CliResult<T> = Result<T, CliError>;
