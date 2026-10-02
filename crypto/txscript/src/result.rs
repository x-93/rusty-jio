use super::error::ScriptError;

pub type ScriptResult<T> = Result<T, ScriptError>;
