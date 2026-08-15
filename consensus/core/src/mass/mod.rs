use crate::block::Block;
use crate::config::params::MAINNET_PARAMS;
use crate::tx::Transaction;

pub const MASS_PER_TX: u64 = 10;
pub const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 1;
pub const MASS_PER_SIG_SCRIPT_BYTE: u64 = 1;
pub const MASS_PER_TX_IN: u64 = 10;
pub const MASS_PER_TX_OUT: u64 = 10;
pub const MASS_PER_PAYLOAD_BYTE: u64 = 1;

pub fn calc_tx_compute_mass(tx: &Transaction) -> u64 {
    if tx.is_coinbase() {
        return 0;
    }

    let mut mass = MASS_PER_TX;

    // Inputs mass
    for input in &tx.inputs {
        mass = mass
            .saturating_add(MASS_PER_TX_IN)
            .saturating_add((input.signature_script.len() as u64) * MASS_PER_SIG_SCRIPT_BYTE);
    }

    // Outputs mass
    for output in &tx.outputs {
        mass = mass
            .saturating_add(MASS_PER_TX_OUT)
            .saturating_add((output.script_public_key.script().len() as u64) * MASS_PER_SCRIPT_PUB_KEY_BYTE);
    }

    // Payload mass
    mass = mass.saturating_add((tx.payload.len() as u64) * MASS_PER_PAYLOAD_BYTE);

    mass
}

pub fn calc_tx_storage_mass(tx: &Transaction) -> u64 {
    if tx.is_coinbase() {
        return 0;
    }
    // Storage mass scales with output count
    (tx.outputs.len() as u64) * 100
}

pub fn calc_tx_mass(tx: &Transaction) -> u64 {
    let compute = calc_tx_compute_mass(tx);
    let storage = calc_tx_storage_mass(tx);
    compute.max(storage)
}

pub fn calc_block_mass(block: &Block) -> u64 {
    block.transactions.iter().map(calc_tx_mass).sum()
}

pub fn check_block_mass(block: &Block, max_mass: Option<u64>) -> bool {
    let limit = max_mass.unwrap_or(MAINNET_PARAMS.max_block_mass);
    calc_block_mass(block) <= limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TransactionInput, TransactionOutpoint, TransactionOutput};
    use jio_hashes::Hash;
    use jio_txscript::ScriptPublicKey;

    #[test]
    fn test_coinbase_mass_zero() {
        let mut coinbase = Transaction::default();
        coinbase.subnetwork_id = crate::subnets::SUBNETWORK_ID_COINBASE;
        assert_eq!(calc_tx_mass(&coinbase), 0);
    }

    #[test]
    fn test_standard_tx_mass() {
        let mut tx = Transaction::default();
        tx.inputs.push(TransactionInput::new(
            TransactionOutpoint::new(Hash::from_bytes([1u8; 32]), 0),
            vec![1, 2, 3, 4],
            0,
            0,
        ));
        tx.outputs.push(TransactionOutput::new(
            1000,
            ScriptPublicKey::new(0, vec![5, 6, 7, 8, 9]),
        ));

        let compute = calc_tx_compute_mass(&tx);
        // Base(10) + Input(10 + 4) + Output(10 + 5) + Payload(0) = 39
        assert_eq!(compute, 39);

        let storage = calc_tx_storage_mass(&tx);
        // 1 output * 100 = 100
        assert_eq!(storage, 100);

        let total_mass = calc_tx_mass(&tx);
        assert_eq!(total_mass, 100);
    }
}
