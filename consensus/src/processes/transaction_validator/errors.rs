use consensus_core::errors::tx::TxRuleError;
use consensus_core::tx::TransactionOutpoint;
use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TxValidationError {
    #[error("Transaction rule error: {0}")]
    Rule(#[from] TxRuleError),

    #[error("Transaction has no inputs")]
    NoInputs,

    #[error("Transaction has no outputs")]
    NoOutputs,

    #[error("Transaction output value overflows u64")]
    OutputValueOverflow,

    #[error("Transaction contains duplicate input: {0:?}")]
    DuplicateInput(TransactionOutpoint),

    #[error("Coinbase payload exceeds maximum allowed size: {0} > {1}")]
    CoinbasePayloadTooLarge(usize, usize),

    #[error("Coinbase transaction cannot have inputs")]
    CoinbaseWithInputs,

    #[error("Block has no coinbase transaction")]
    MissingCoinbase,

    #[error("Coinbase transaction is not the first transaction")]
    CoinbaseNotFirst,

    #[error("Multiple coinbase transactions in block")]
    MultipleCoinbases,

    #[error("Block contains duplicate transaction {0}")]
    DuplicateTransactionInBlock(Hash),

    #[error("Block transaction mass {0} exceeds max allowed {1}")]
    ExceedsBlockMass(u64, u64),

    #[error("Total output value {0} exceeds total input value {1}")]
    SpendExceedsInputs(u64, u64),
}

pub type TxValidationResult<T> = Result<T, TxValidationError>;
