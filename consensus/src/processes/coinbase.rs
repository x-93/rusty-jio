use jio_consensus_core::coinbase::{
    BlockRewardData, CoinbaseData, CoinbaseTransactionTemplate, MinerData,
};
use jio_consensus_core::config::params::Params;
use jio_consensus_core::constants::TX_VERSION;
use jio_consensus_core::errors::coinbase::CoinbaseError;
use jio_consensus_core::subnets::SUBNETWORK_ID_COINBASE;
use jio_consensus_core::tx::{
    ScriptPublicKey, Transaction, TransactionInput, TransactionOutput, TransactionOutpoint,
};
use jio_hashes::Hash;

pub fn serialize_coinbase_payload(
    blue_score: u64,
    spk: &ScriptPublicKey,
    extra_data: &[u8],
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(8 + 2 + spk.script().len() + extra_data.len());
    payload.extend_from_slice(&blue_score.to_le_bytes());
    payload.extend_from_slice(&spk.version().to_le_bytes());
    payload.extend_from_slice(&(spk.script().len() as u8).to_le_bytes());
    payload.extend_from_slice(spk.script());
    payload.extend_from_slice(extra_data);
    payload
}

pub fn deserialize_coinbase_payload(
    payload: &[u8],
) -> Result<CoinbaseData<Vec<u8>>, CoinbaseError> {
    if payload.len() < 11 {
        return Err(CoinbaseError::PayloadTooShort(payload.len()));
    }

    let blue_score = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let version = u16::from_le_bytes(payload[8..10].try_into().unwrap());
    let script_len = payload[10] as usize;

    if payload.len() < 11 + script_len {
        return Err(CoinbaseError::PayloadTooShort(payload.len()));
    }

    let script = payload[11..11 + script_len].to_vec();
    let extra_data = payload[11 + script_len..].to_vec();

    Ok(CoinbaseData {
        blue_score,
        subsidy: 0,
        miner_data: MinerData::new(ScriptPublicKey::new(version, script), extra_data),
    })
}

#[derive(Clone)]
pub struct CoinbaseManager {
    params: Params,
}

impl CoinbaseManager {
    pub fn new(params: Params) -> Self {
        Self { params }
    }

    /// Calculates the block subsidy based on DAA score following pre-deflationary and deflationary phases
    pub fn calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.params.deflationary_phase_daa_score {
            self.params.pre_deflationary_phase_base_subsidy
        } else {
            let deflationary_score = daa_score - self.params.deflationary_phase_daa_score;
            let halving_interval = 12 * 30 * 24 * 60 * 60 * self.params.bps();
            let halvings = if halving_interval > 0 {
                deflationary_score / halving_interval
            } else {
                0
            };
            if halvings >= 64 {
                0
            } else {
                self.params.pre_deflationary_phase_base_subsidy >> (halvings + 1)
            }
        }
    }

    /// Creates a coinbase transaction template given blue score, miner data, and block rewards
    pub fn create_coinbase_transaction(
        &self,
        blue_score: u64,
        miner_data: &MinerData,
        rewards: Vec<BlockRewardData>,
    ) -> Result<CoinbaseTransactionTemplate, CoinbaseError> {
        let payload = serialize_coinbase_payload(
            blue_score,
            &miner_data.script_public_key,
            &miner_data.extra_data,
        );

        if payload.len() > self.params.max_coinbase_payload_len {
            return Err(CoinbaseError::PayloadTooLong(
                payload.len(),
                self.params.max_coinbase_payload_len,
            ));
        }

        let mut outputs = Vec::with_capacity(rewards.len());
        for reward in rewards {
            let amount = reward.subsidy.saturating_add(reward.total_fees);
            if amount > 0 {
                outputs.push(TransactionOutput::new(amount, reward.script_public_key));
            }
        }

        // If no explicit outputs were passed, default to one output with block subsidy to miner
        if outputs.is_empty() {
            let subsidy = self.calc_block_subsidy(blue_score);
            outputs.push(TransactionOutput::new(subsidy, miner_data.script_public_key.clone()));
        }

        let tx = Transaction {
            version: TX_VERSION,
            inputs: vec![TransactionInput {
                previous_outpoint: TransactionOutpoint::new(Hash::default(), u32::MAX),
                signature_script: Vec::new(),
                sequence: 0,
                sig_op_count: 0,
            }],
            outputs,
            lock_time: 0,
            subnetwork_id: SUBNETWORK_ID_COINBASE,
            gas: 0,
            payload,
            mass: 0,
        };

        Ok(CoinbaseTransactionTemplate {
            tx,
            has_red_reward: false,
        })
    }

    /// Validates that a coinbase transaction conforms to all consensus rules
    pub fn validate_coinbase_transaction(
        &self,
        tx: &Transaction,
        blue_score: u64,
        expected_subsidy: u64,
    ) -> Result<CoinbaseData<Vec<u8>>, CoinbaseError> {
        if !tx.is_coinbase() {
            return Err(CoinbaseError::NotCoinbase);
        }

        let coinbase_data = deserialize_coinbase_payload(&tx.payload)?;
        if coinbase_data.blue_score != blue_score {
            return Err(CoinbaseError::BlueScoreMismatch(
                coinbase_data.blue_score,
                blue_score,
            ));
        }

        if tx.payload.len() > self.params.max_coinbase_payload_len {
            return Err(CoinbaseError::PayloadTooLong(
                tx.payload.len(),
                self.params.max_coinbase_payload_len,
            ));
        }

        let total_value: u64 = tx.outputs.iter().map(|o| o.value).sum();
        if total_value < expected_subsidy {
            return Err(CoinbaseError::InsufficientSubsidy(
                total_value,
                expected_subsidy,
            ));
        }

        Ok(coinbase_data)
    }
}

pub fn calc_block_subsidy(daa_score: u64) -> u64 {
    let manager = CoinbaseManager::new(Params::mainnet());
    manager.calc_block_subsidy(daa_score)
}

pub fn create_coinbase_transaction(
    blue_score: u64,
    subsidy: u64,
    spk: ScriptPublicKey,
) -> Transaction {
    let manager = CoinbaseManager::new(Params::mainnet());
    let miner_data = MinerData::new(spk, vec![]);
    let reward = BlockRewardData::new(subsidy, 0, miner_data.script_public_key.clone());
    manager.create_coinbase_transaction(blue_score, &miner_data, vec![reward]).unwrap().tx
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_txscript::ScriptPublicKey;

    #[test]
    fn test_coinbase_manager_subsidy_lifecycle() {
        let manager = CoinbaseManager::new(Params::mainnet());
        let pre_deflationary = manager.calc_block_subsidy(100);
        assert_eq!(pre_deflationary, 50_000_000_000);

        let deflationary = manager.calc_block_subsidy(Params::mainnet().deflationary_phase_daa_score + 100);
        assert_eq!(deflationary, 25_000_000_000);
    }

    #[test]
    fn test_coinbase_manager_create_and_validate() {
        let manager = CoinbaseManager::new(Params::mainnet());
        let miner_spk = ScriptPublicKey::new(0, vec![1, 2, 3, 4]);
        let miner_data = MinerData::new(miner_spk.clone(), b"rusty-jio".to_vec());
        let blue_score = 42;

        let reward1 = BlockRewardData::new(25_000_000_000, 1000, miner_spk.clone());
        let reward2 = BlockRewardData::new(25_000_000_000, 2000, ScriptPublicKey::new(0, vec![5, 6, 7]));

        let template = manager
            .create_coinbase_transaction(blue_score, &miner_data, vec![reward1, reward2])
            .unwrap();

        assert!(template.tx.is_coinbase());
        assert_eq!(template.tx.outputs.len(), 2);
        assert_eq!(template.tx.outputs[0].value, 25_000_001_000);
        assert_eq!(template.tx.outputs[1].value, 25_000_002_000);

        let validated = manager
            .validate_coinbase_transaction(&template.tx, blue_score, 50_000_000_000)
            .unwrap();

        assert_eq!(validated.blue_score, blue_score);
        assert_eq!(validated.miner_data.script_public_key, miner_spk);
        assert_eq!(validated.miner_data.extra_data, b"rusty-jio".to_vec());
    }

    #[test]
    fn test_coinbase_manager_validation_rejections() {
        let manager = CoinbaseManager::new(Params::mainnet());
        let miner_spk = ScriptPublicKey::new(0, vec![1, 2, 3]);
        let miner_data = MinerData::new(miner_spk, vec![]);
        let template = manager.create_coinbase_transaction(10, &miner_data, vec![]).unwrap();

        // 1. Wrong blue score
        let err = manager.validate_coinbase_transaction(&template.tx, 999, 100).unwrap_err();
        match err {
            CoinbaseError::BlueScoreMismatch(actual, expected) => {
                assert_eq!(actual, 10);
                assert_eq!(expected, 999);
            }
            other => panic!("expected BlueScoreMismatch, got {other:?}"),
        }

        // 2. Insufficient subsidy
        let err = manager.validate_coinbase_transaction(&template.tx, 10, 100_000_000_000).unwrap_err();
        match err {
            CoinbaseError::InsufficientSubsidy(_, _) => {}
            other => panic!("expected InsufficientSubsidy, got {other:?}"),
        }
    }
}


