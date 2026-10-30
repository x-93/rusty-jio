use crate::tx::Transaction;
use crate::utxo::UtxoEntry;
use jio_hashes::{Hash, Hasher, TransactionSigningHash};

pub const SIG_HASH_ALL: u8 = 0b00000001;
pub const SIG_HASH_NONE: u8 = 0b00000010;
pub const SIG_HASH_SINGLE: u8 = 0b00000100;
pub const SIG_HASH_ANYONECANPAY: u8 = 0b10000000;

/// Precomputes and caches common transaction hashes across all inputs.
/// Drops sighash verification from quadratic O(N^2) to linear O(N).
#[derive(Clone, Debug, Default)]
pub struct SigHashReusedValues {
    pub previous_outputs_hash: Option<Hash>,
    pub sequences_hash: Option<Hash>,
    pub sig_op_counts_hash: Option<Hash>,
    pub payload_hash: Option<Hash>,
    pub outputs_hash: Option<Hash>,
}

impl SigHashReusedValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tx(tx: &Transaction) -> Self {
        let mut reused = Self::new();

        // 1. Previous outputs hash
        let mut hasher = TransactionSigningHash::new();
        for input in &tx.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(input.previous_outpoint.index.to_le_bytes());
        }
        reused.previous_outputs_hash = Some(hasher.finalize());

        // 2. Sequences hash
        let mut hasher = TransactionSigningHash::new();
        for input in &tx.inputs {
            hasher.update(input.sequence.to_le_bytes());
        }
        reused.sequences_hash = Some(hasher.finalize());

        // 3. SigOp counts hash
        let mut hasher = TransactionSigningHash::new();
        for input in &tx.inputs {
            hasher.update([input.sig_op_count]);
        }
        reused.sig_op_counts_hash = Some(hasher.finalize());

        // 4. Payload hash
        let mut hasher = TransactionSigningHash::new();
        hasher.update(tx.subnetwork_id.0);
        hasher.update(tx.gas.to_le_bytes());
        let payload_len = tx.payload.len() as u32;
        hasher.update(payload_len.to_le_bytes());
        hasher.update(&tx.payload);
        reused.payload_hash = Some(hasher.finalize());

        // 5. Outputs hash
        let mut hasher = TransactionSigningHash::new();
        for output in &tx.outputs {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(script_len.to_le_bytes());
            hasher.update(&output.script_public_key.script);
        }
        reused.outputs_hash = Some(hasher.finalize());

        reused
    }
}

/// Computes the Schnorr signature hash for a specific transaction input.
pub fn calc_schnorr_signature_hash(
    tx: &Transaction,
    input_index: usize,
    utxo_entry: &UtxoEntry,
    hash_type: u8,
    reused: &SigHashReusedValues,
) -> Hash {
    let mut hasher = TransactionSigningHash::new();

    // 1. Version
    hasher.update(tx.version.to_le_bytes());

    // 2. Previous outputs hash (if not ANYONECANPAY)
    if hash_type & SIG_HASH_ANYONECANPAY == 0 {
        if let Some(h) = reused.previous_outputs_hash {
            hasher.update(h.as_bytes());
        }
    }

    // 3. Sequences hash (if not ANYONECANPAY and not SINGLE/NONE)
    if hash_type & SIG_HASH_ANYONECANPAY == 0
        && (hash_type & 0x07) != SIG_HASH_SINGLE
        && (hash_type & 0x07) != SIG_HASH_NONE
    {
        if let Some(h) = reused.sequences_hash {
            hasher.update(h.as_bytes());
        }
    }

    // 4. SigOp counts hash
    if hash_type & SIG_HASH_ANYONECANPAY == 0 {
        if let Some(h) = reused.sig_op_counts_hash {
            hasher.update(h.as_bytes());
        }
    }

    // 5. Input specific data
    let input = &tx.inputs[input_index];
    hasher.update(input.previous_outpoint.transaction_id.as_bytes());
    hasher.update(input.previous_outpoint.index.to_le_bytes());

    // 6. UTXO being spent
    hasher.update(utxo_entry.amount.to_le_bytes());
    hasher.update(utxo_entry.script_public_key.version.to_le_bytes());
    let script_len = utxo_entry.script_public_key.script.len() as u32;
    hasher.update(script_len.to_le_bytes());
    hasher.update(&utxo_entry.script_public_key.script);

    // 7. Input sequence and sig_op_count
    hasher.update(input.sequence.to_le_bytes());
    hasher.update([input.sig_op_count]);

    // 8. Outputs hash
    if (hash_type & 0x07) != SIG_HASH_NONE && (hash_type & 0x07) != SIG_HASH_SINGLE {
        if let Some(h) = reused.outputs_hash {
            hasher.update(h.as_bytes());
        }
    } else if (hash_type & 0x07) == SIG_HASH_SINGLE && input_index < tx.outputs.len() {
        let mut single_hasher = TransactionSigningHash::new();
        let output = &tx.outputs[input_index];
        single_hasher.update(output.value.to_le_bytes());
        single_hasher.update(output.script_public_key.version.to_le_bytes());
        let script_len = output.script_public_key.script.len() as u32;
        single_hasher.update(script_len.to_le_bytes());
        single_hasher.update(&output.script_public_key.script);
        hasher.update(single_hasher.finalize().as_bytes());
    }

    // 9. Lock time, Subnetwork, and Hash type
    hasher.update(tx.lock_time.to_le_bytes());
    hasher.update(tx.subnetwork_id.0);
    hasher.update(tx.gas.to_le_bytes());
    if let Some(h) = reused.payload_hash {
        hasher.update(h.as_bytes());
    }
    hasher.update([hash_type]);

    hasher.finalize()
}
