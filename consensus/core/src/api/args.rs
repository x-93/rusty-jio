use crate::block::Block;
use crate::tx::Transaction;

#[derive(Clone, Debug)]
pub struct SubmitBlockArgs {
    pub block: Block,
}

#[derive(Clone, Debug)]
pub struct ValidateTxArgs {
    pub transaction: Transaction,
}
