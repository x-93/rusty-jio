use crate::constants::STORAGE_MASS_PARAMETER;
use crate::tx::Transaction;

pub const DEFAULT_MASS_PER_TX_BYTE: u64 = 1;
pub const DEFAULT_MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
pub const DEFAULT_MASS_PER_SIG_OP: u64 = 1000;

#[derive(Clone, Copy, Debug)]
pub struct MassCalculator {
    pub mass_per_tx_byte: u64,
    pub mass_per_script_pub_key_byte: u64,
    pub mass_per_sig_op: u64,
    pub storage_mass_parameter: u64,
}

impl Default for MassCalculator {
    fn default() -> Self {
        Self::new(
            DEFAULT_MASS_PER_TX_BYTE,
            DEFAULT_MASS_PER_SCRIPT_PUB_KEY_BYTE,
            DEFAULT_MASS_PER_SIG_OP,
            STORAGE_MASS_PARAMETER,
        )
    }
}

impl MassCalculator {
    pub fn new(
        mass_per_tx_byte: u64,
        mass_per_script_pub_key_byte: u64,
        mass_per_sig_op: u64,
        storage_mass_parameter: u64,
    ) -> Self {
        Self {
            mass_per_tx_byte,
            mass_per_script_pub_key_byte,
            mass_per_sig_op,
            storage_mass_parameter,
        }
    }

    pub fn calc_compute_mass(&self, tx: &Transaction) -> u64 {
        let mut mass = (tx.inputs.len() as u64) * 1000 + (tx.outputs.len() as u64) * 1000 + (tx.payload.len() as u64);
        for input in &tx.inputs {
            mass += (input.sig_op_count as u64) * self.mass_per_sig_op;
        }
        mass
    }

    pub fn calc_storage_mass(&self, output_value: u64) -> u64 {
        self.storage_mass_parameter
            .checked_div(output_value)
            .unwrap_or(self.storage_mass_parameter)
    }

    pub fn calc_overall_mass(&self, tx: &Transaction) -> u64 {
        let compute_mass = self.calc_compute_mass(tx);
        let mut storage_mass = 0;
        for output in &tx.outputs {
            storage_mass += self.calc_storage_mass(output.value);
        }
        compute_mass.max(storage_mass)
    }
}
