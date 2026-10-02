//! Transaction mass and fee rate calculation.

use super::tx::Transaction;

pub const MASS_PER_BYTE: u64 = 1;
pub const MASS_PER_SIG_OP: u64 = 1000;

/// Calculates transaction mass based on byte size and signature operation count.
pub fn calc_tx_mass(tx: &Transaction) -> u64 {
    let mut bytes_len = 2; // version
    bytes_len += 8; // input len
    for input in &tx.inputs {
        bytes_len += 32 + 4 + 8 + 1 + input.signature_script.len();
    }
    bytes_len += 8; // output len
    for output in &tx.outputs {
        bytes_len += 8 + 2 + output.script_public_key.script.len();
    }
    bytes_len += 8 + 20 + 8 + tx.payload.len();

    let byte_mass = (bytes_len as u64) * MASS_PER_BYTE;
    let mut sig_ops = 0u64;
    for input in &tx.inputs {
        sig_ops += input.sig_op_count as u64;
    }
    let sigop_mass = sig_ops * MASS_PER_SIG_OP;

    byte_mass.max(sigop_mass)
}
