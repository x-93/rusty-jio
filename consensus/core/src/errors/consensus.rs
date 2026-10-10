use super::block::BlockRuleError;
use super::tx::TxRuleError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ConsensusError {
    #[error("block rule error: {0}")]
    BlockRuleError(#[from] BlockRuleError),

    #[error("tx rule error: {0}")]
    TxRuleError(#[from] TxRuleError),

    #[error("consensus error: {0}")]
    General(String),
}
