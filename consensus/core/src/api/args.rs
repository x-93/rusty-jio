use crate::coinbase::MinerData;

pub struct TransactionValidationBatchArgs {
    pub block_daa_score: u64,
}

pub struct BuildBlockTemplateArgs {
    pub miner_data: MinerData,
}
