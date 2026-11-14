use crate::tx::Transaction;

pub const MASS_PER_TX_BYTE: u64 = 1;
pub const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
pub const MASS_PER_SIG_OP: u64 = 1000;

#[derive(Clone, Copy, Debug, Default)]
pub struct TxMassCalculator {
    mass_per_tx_byte: u64,
    mass_per_script_pub_key_byte: u64,
    mass_per_sig_op: u64,
}

impl TxMassCalculator {
    pub const fn new(
        mass_per_tx_byte: u64,
        mass_per_script_pub_key_byte: u64,
        mass_per_sig_op: u64,
    ) -> Self {
        Self {
            mass_per_tx_byte,
            mass_per_script_pub_key_byte,
            mass_per_sig_op,
        }
    }

    /// Calculates compute mass: size in bytes plus signature operations cost.
    pub fn calc_compute_mass(&self, tx: &Transaction) -> u64 {
        if tx.is_coinbase() {
            return 0;
        }

        let mut size: u64 = 0;
        // Version (2) + LockTime (8) + SubnetworkId (20) + Gas (8) + Payload length (8)
        size += 2 + 8 + 20 + 8 + 8 + tx.payload.len() as u64;

        let mut total_sig_ops: u64 = 0;
        for input in &tx.inputs {
            // Previous outpoint (36) + Sequence (8) + SigOpCount (1) + sig script len (8)
            size += 36 + 8 + 1 + 8 + input.signature_script.len() as u64;
            total_sig_ops += input.sig_op_count as u64;
        }

        let mut script_pub_key_bytes: u64 = 0;
        for output in &tx.outputs {
            // Value (8) + script version (2) + script len (8)
            size += 8 + 2 + 8;
            script_pub_key_bytes += output.script_public_key.script.len() as u64;
        }

        size * self.mass_per_tx_byte
            + script_pub_key_bytes * self.mass_per_script_pub_key_byte
            + total_sig_ops * self.mass_per_sig_op
    }

    /// Overall transaction mass
    pub fn calc_tx_mass(&self, tx: &Transaction) -> u64 {
        self.calc_compute_mass(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subnets::SubnetworkId;

    #[test]
    fn test_mass_calculator() {
        let calc = TxMassCalculator::new(
            MASS_PER_TX_BYTE,
            MASS_PER_SCRIPT_PUB_KEY_BYTE,
            MASS_PER_SIG_OP,
        );
        let tx = Transaction::new(1, vec![], vec![], 0, SubnetworkId::NATIVE, 0, vec![]);
        assert!(calc.calc_tx_mass(&tx) > 0);

        let coinbase = Transaction::new(1, vec![], vec![], 0, SubnetworkId::COINBASE, 0, vec![]);
        assert_eq!(calc.calc_tx_mass(&coinbase), 0);
    }
}
