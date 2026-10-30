use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, Hasher, TransactionHash, TransactionID};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct TransactionOutpoint {
    pub transaction_id: Hash,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: Hash, index: u32) -> Self {
        Self {
            transaction_id,
            index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: Vec<u8>,
}

impl ScriptPublicKey {
    pub fn new(version: u16, script: Vec<u8>) -> Self {
        Self { version, script }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransactionInput {
    pub previous_outpoint: TransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct SubnetworkId(pub [u8; 20]);

impl SubnetworkId {
    pub const NATIVE: SubnetworkId = SubnetworkId([0u8; 20]);
    pub const COINBASE: SubnetworkId =
        SubnetworkId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
    pub mass: u64,
    pub id: Hash,
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
        let mut tx = Self {
            version,
            inputs,
            outputs,
            lock_time,
            subnetwork_id,
            gas,
            payload,
            mass: 0,
            id: Hash::ZERO,
        };
        tx.id = tx.compute_id();
        tx
    }

    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id == SubnetworkId::COINBASE
    }

    /// Transaction ID excludes signature scripts to prevent transaction malleability.
    pub fn compute_id(&self) -> Hash {
        let mut hasher = TransactionID::new();
        hasher.update(self.version.to_le_bytes());

        let num_inputs = self.inputs.len() as u32;
        hasher.update(num_inputs.to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(input.previous_outpoint.index.to_le_bytes());
            hasher.update(input.sequence.to_le_bytes());
        }

        let num_outputs = self.outputs.len() as u32;
        hasher.update(num_outputs.to_le_bytes());
        for output in &self.outputs {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(script_len.to_le_bytes());
            hasher.update(&output.script_public_key.script);
        }

        hasher.update(self.lock_time.to_le_bytes());
        hasher.update(self.subnetwork_id.0);
        hasher.update(self.gas.to_le_bytes());
        let payload_len = self.payload.len() as u32;
        hasher.update(payload_len.to_le_bytes());
        hasher.update(&self.payload);

        hasher.finalize()
    }

    /// Full transaction hash includes signature scripts.
    pub fn compute_hash(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(self.version.to_le_bytes());

        let num_inputs = self.inputs.len() as u32;
        hasher.update(num_inputs.to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(input.previous_outpoint.index.to_le_bytes());
            let sig_len = input.signature_script.len() as u32;
            hasher.update(sig_len.to_le_bytes());
            hasher.update(&input.signature_script);
            hasher.update(input.sequence.to_le_bytes());
            hasher.update([input.sig_op_count]);
        }

        let num_outputs = self.outputs.len() as u32;
        hasher.update(num_outputs.to_le_bytes());
        for output in &self.outputs {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(script_len.to_le_bytes());
            hasher.update(&output.script_public_key.script);
        }

        hasher.update(self.lock_time.to_le_bytes());
        hasher.update(self.subnetwork_id.0);
        hasher.update(self.gas.to_le_bytes());
        let payload_len = self.payload.len() as u32;
        hasher.update(payload_len.to_le_bytes());
        hasher.update(&self.payload);

        hasher.finalize()
    }
}
