use crate::constants::TX_VERSION;
use crate::subnets::{SubnetworkId, SUBNETWORK_ID_COINBASE, SUBNETWORK_ID_NATIVE};
use jio_hashes::Hash;
pub use jio_txscript::ScriptPublicKey;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type TransactionId = Hash;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionOutpoint {
    pub transaction_id: TransactionId,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: TransactionId, index: u32) -> Self {
        Self {
            transaction_id,
            index,
        }
    }
}

impl fmt::Display for TransactionOutpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.transaction_id, self.index)
    }
}

impl fmt::Debug for TransactionOutpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Outpoint({}:{})", self.transaction_id, self.index)
    }
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct TransactionInput {
    pub previous_outpoint: TransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
}

impl TransactionInput {
    pub fn new(
        previous_outpoint: TransactionOutpoint,
        signature_script: Vec<u8>,
        sequence: u64,
        sig_op_count: u8,
    ) -> Self {
        Self {
            previous_outpoint,
            signature_script,
            sequence,
            sig_op_count,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TransactionOutput {
    pub fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self {
            value,
            script_public_key,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
    pub mass: u64,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            version: TX_VERSION,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
            subnetwork_id: SUBNETWORK_ID_NATIVE,
            gas: 0,
            payload: Vec::new(),
            mass: 0,
        }
    }
}

impl Transaction {
    pub fn new(
        version: u16,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            lock_time,
            subnetwork_id,
            gas,
            payload,
            mass: 0,
        }
    }

    pub fn id(&self) -> TransactionId {
        crate::hashing::tx::tx_id(self)
    }

    pub fn hash(&self) -> Hash {
        crate::hashing::tx::tx_hash(self)
    }

    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id == SUBNETWORK_ID_COINBASE
    }

    pub fn total_out_value(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }

    pub fn calc_mass(&self) -> u64 {
        if self.mass > 0 {
            return self.mass;
        }
        let input_mass = self.inputs.iter().map(|i| 1000 + i.signature_script.len() as u64).sum::<u64>();
        let output_mass = self.outputs.iter().map(|o| 1000 + o.script_public_key.script().len() as u64).sum::<u64>();
        let payload_mass = self.payload.len() as u64;
        (input_mass + output_mass + payload_mass).max(1)
    }

    pub fn with_mass(mut self, mass: u64) -> Self {
        self.mass = mass;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutableTransaction {
    pub tx: Transaction,
    pub entries: Vec<Option<crate::utxo::UtxoEntry>>,
    pub calculated_fee: Option<u64>,
    pub calculated_mass: Option<u64>,
}

impl MutableTransaction {
    pub fn new(tx: Transaction) -> Self {
        let entries = vec![None; tx.inputs.len()];
        Self {
            tx,
            entries,
            calculated_fee: None,
            calculated_mass: None,
        }
    }

    pub fn from_tx(tx: Transaction) -> Self {
        Self::new(tx)
    }

    pub fn with_entries(tx: Transaction, entries: Vec<Option<crate::utxo::UtxoEntry>>) -> Self {
        Self {
            tx,
            entries,
            calculated_fee: None,
            calculated_mass: None,
        }
    }

    pub fn id(&self) -> TransactionId {
        self.tx.id()
    }

    pub fn hash(&self) -> Hash {
        self.tx.hash()
    }

    pub fn as_ref(&self) -> &Transaction {
        &self.tx
    }

    pub fn into_tx(self) -> Transaction {
        self.tx
    }

    pub fn total_in_value(&self) -> Option<u64> {
        let mut total = 0u64;
        for entry in &self.entries {
            match entry {
                Some(e) => total = total.checked_add(e.amount)?,
                None => return None,
            }
        }
        Some(total)
    }

    pub fn total_out_value(&self) -> u64 {
        self.tx.total_out_value()
    }

    pub fn calculate_fee(&mut self) -> Option<u64> {
        let total_in = self.total_in_value()?;
        let total_out = self.total_out_value();
        if total_in >= total_out {
            let fee = total_in - total_out;
            self.calculated_fee = Some(fee);
            Some(fee)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utxo::UtxoEntry;

    #[test]
    fn test_tx_primitives_and_mutable_tx() {
        let mut tx = Transaction::default();
        tx.inputs.push(TransactionInput::new(
            TransactionOutpoint::new(Hash::from_bytes([1u8; 32]), 0),
            vec![],
            0,
            0,
        ));
        tx.outputs.push(TransactionOutput::new(100, ScriptPublicKey::default()));

        let id = tx.id();
        assert_ne!(id, Hash::default());
        assert_eq!(tx.total_out_value(), 100);

        let mut mtx = MutableTransaction::from_tx(tx.clone());
        assert_eq!(mtx.total_in_value(), None);

        // Populate UTXO entry
        mtx.entries = vec![Some(UtxoEntry::new(150, ScriptPublicKey::default(), 1, false))];
        assert_eq!(mtx.total_in_value(), Some(150));
        assert_eq!(mtx.calculate_fee(), Some(50));
        assert_eq!(mtx.calculated_fee, Some(50));
    }
}
