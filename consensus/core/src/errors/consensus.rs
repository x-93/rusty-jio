use crate::errors::block::BlockRuleError;
use crate::errors::coinbase::CoinbaseError;
use crate::errors::config::ConfigError;
use crate::errors::difficulty::DifficultyError;
use crate::errors::pruning::PruningError;
use crate::errors::sync::SyncError;
use crate::errors::traversal::TraversalError;
use crate::errors::tx::TxRuleError;
use crate::utxo::UtxoError;
use jio_addresses::AddressError;
use jio_txscript::TxScriptError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ConsensusError {
    #[error("block rule error: {0}")]
    BlockRule(#[from] BlockRuleError),
    #[error("transaction rule error: {0}")]
    TxRule(#[from] TxRuleError),
    #[error("coinbase error: {0}")]
    Coinbase(#[from] CoinbaseError),
    #[error("difficulty error: {0}")]
    Difficulty(#[from] DifficultyError),
    #[error("pruning error: {0}")]
    Pruning(#[from] PruningError),
    #[error("sync error: {0}")]
    Sync(#[from] SyncError),
    #[error("traversal error: {0}")]
    Traversal(#[from] TraversalError),
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("utxo error: {0}")]
    Utxo(#[from] UtxoError),
    #[error("address error: {0}")]
    Address(#[from] AddressError),
    #[error("txscript error: {0}")]
    TxScript(#[from] TxScriptError),
    #[error("general consensus error: {0}")]
    General(String),
}

pub type ConsensusResult<T> = Result<T, ConsensusError>;
pub type BlockResult<T> = Result<T, BlockRuleError>;
pub type TxResult<T> = Result<T, TxRuleError>;
pub type UtxoResult<T> = Result<T, UtxoError>;
pub type DifficultyResult<T> = Result<T, DifficultyError>;
pub type CoinbaseResult<T> = Result<T, CoinbaseError>;
pub type ConfigResult<T> = Result<T, ConfigError>;

