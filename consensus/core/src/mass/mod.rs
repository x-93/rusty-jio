use crate::tx::Transaction;
use crate::utxo::UtxoEntry;

pub const MASS_PER_TX_BYTE: u64 = 1;
pub const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
pub const MASS_PER_SIG_OP: u64 = 1000;
pub const STORAGE_MASS_PARAMETER: u64 = 10_000_000_000; // C = 10^10 Sompis

/// Calculates compute mass based on serialized bytes and signature operations.
pub fn calc_compute_mass(tx: &Transaction) -> u64 {
    let mut bytes = 4; // version (2) + lock_time (2)
    bytes += 4; // inputs len
    for input in &tx.inputs {
        bytes += 32 + 4 + 8 + 1 + input.signature_script.len() as u64;
    }
    bytes += 4; // outputs len
    for output in &tx.outputs {
        bytes += 8 + 2 + 4 + output.script_public_key.script.len() as u64;
    }
    bytes += 20 + 8 + 4 + tx.payload.len() as u64;

    let mut sig_ops: u64 = 0;
    for input in &tx.inputs {
        sig_ops += input.sig_op_count as u64;
    }

    bytes * MASS_PER_TX_BYTE + sig_ops * MASS_PER_SIG_OP
}

/// Calculates storage mass according to KIP-0009:
/// StorageMass = max(0, C * ( sum(1 / v_out) - sum(1 / v_in) ))
/// Dust outputs with tiny values produce large storage mass penalties!
pub fn calc_storage_mass(tx: &Transaction, utxo_entries: &[UtxoEntry]) -> u64 {
    if tx.is_coinbase() {
        return 0;
    }

    let mut out_harmonic_sum: f64 = 0.0;
    for out in &tx.outputs {
        let val = out.value.max(1) as f64;
        out_harmonic_sum += (STORAGE_MASS_PARAMETER as f64) / val;
    }

    let mut in_harmonic_sum: f64 = 0.0;
    for entry in utxo_entries {
        let val = entry.amount.max(1) as f64;
        in_harmonic_sum += (STORAGE_MASS_PARAMETER as f64) / val;
    }

    let diff = out_harmonic_sum - in_harmonic_sum;
    if diff > 0.0 {
        diff.ceil() as u64
    } else {
        0
    }
}

/// Overall transaction mass is max(compute_mass, storage_mass).
pub fn calc_overall_mass(tx: &Transaction, utxo_entries: &[UtxoEntry]) -> u64 {
    let compute = calc_compute_mass(tx);
    let storage = calc_storage_mass(tx, utxo_entries);
    compute.max(storage)
}
