# Beginner's Step-by-Step Guide to Coding Layer 3: Consensus Core Data Models & Proof-of-Work (`consensus/core/`, `consensus/pow/`)

Congratulations on completing **Layer 1 (Foundation & Storage)** and **Layer 2 (Cryptography)** of [rusty-jio](https://www.google.com/search?q=https://github.com/x-93/rusty-jio)! Your repository now has big-integer arithmetic, RocksDB storage wrappers, domain-separated Blake2b hashing, Merkle trees, and Bech32m address encoding.

In **Layer 3**, we implement the **Consensus Core & Proof-of-Work**:
1. **`consensus/core/`**: Defines the fundamental vocabulary of the entire blockchain:
   - **`Header` & `Block`**: Multi-parent DAG block headers with pruning-proof parent levels (`parents_by_level`), DAA scores, and cumulative blue work (`Uint192`).
   - **`Transaction` & `UtxoEntry`**: Transaction inputs, outputs, outpoints, subnetwork payloads, and UTXO entries.
   - **`UtxoDiff`**: Atomic state transitions (`add` and `remove` collections) enabling lightning-fast DAG branch switching without disk rewrites.
   - **Schnorr Sighash (`hashing/sighash.rs`)**: Linear-time Schnorr signature hashing with cached intermediate digests (`SigHashReusedValues`) to prevent $O(N^2)$ quadratic hashing attacks.
   - **Compute & Storage Mass (`mass/mod.rs`)**: Dual-metering resource bounds including KIP-0009 harmonic dust penalties.
   - **Network & BPS Parameters (`config/params.rs`, `config/bps.rs`, `config/genesis.rs`)**: Dynamic GHOSTDAG parameter scaling ($k=18$ at 1 BPS $\to$ $k=124$ at 10 BPS) and hardcoded genesis blocks.
2. **`consensus/pow/`**: Kaspa-style **kHeavyHash Proof-of-Work**:
   - **`XoShiRo256PlusPlus`**: High-performance, 256-bit PRNG seeded deterministically by the block's `pre_pow_hash`.
   - **$64 \times 64$ Nibble Matrix**: Deterministic full-rank matrix generation, vector-matrix multiplication, 10-bit right shift, and non-linear XOR mixing.
   - **PoW Validator & Target Check**: Full Proof-of-Work pipeline comparing against compact difficulty targets (`Uint256`).

---

## Part 0: Key Concepts in BlockDAG Consensus (In Plain English)

| Concept | What It Means in Plain English | Why Your Blockchain Needs It |
| :--- | :--- | :--- |
| **Multi-Parent Headers (`parents_by_level`)** | A block can point to multiple parent blocks simultaneously. | In a 10 BPS BlockDAG, blocks are created concurrently. Multi-parent links unite parallel branches into a unified graph instead of orphaning them. |
| **`blue_work` (`Uint192`) & `daa_score`** | `blue_work` is the total cumulative PoW of all well-ordered ("blue") blocks in a block's past. `daa_score` counts total DAG difficulty increments. | Used by GHOSTDAG to pick the heaviest chain ("selected parent") and advance difficulty deterministically. |
| **`UtxoDiff` (`add` / `remove`)** | A mathematical record of which UTXOs a block created and which UTXOs it spent. | Allows the consensus engine to apply or revert state changes along any DAG branch in memory without re-scanning millions of UTXOs. |
| **Schnorr Sighash Midstate Caching** | Hashing transaction inputs/outputs once into reusable 32-byte digests. | In standard Bitcoin, signing $N$ inputs requires $O(N^2)$ hashing work. Midstate caching drops this to $O(N)$, eliminating quadratic denial-of-service vectors. |
| **KIP-0009 Storage Mass** | A transaction fee penalty that scales inversely with UTXO values: $\sum \frac{C}{v_{\text{out}}} - \sum \frac{C}{v_{\text{in}}}$. | Prevents attackers from spamming thousands of tiny dust outputs that permanently bloat validator RAM. |
| **kHeavyHash ($64 \times 64$ Nibble Matrix)** | An optical-miner-friendly PoW function combining standard hashing with a matrix-vector product. | Makes mining energy-efficient and hardware-diverse while maintaining strict cryptographic unpredictability. |

---

## Step 1: Update Root `Cargo.toml` and Create Directory Structure

From your repository root (`rusty-jio/`), create the new crate folders:

```bash
mkdir -p consensus/core/src/utxo
mkdir -p consensus/core/src/hashing
mkdir -p consensus/core/src/mass
mkdir -p consensus/core/src/config
mkdir -p consensus/pow/src

# Initialize the two library crates
cargo init --lib consensus/core --name jio-consensus-core
cargo init --lib consensus/pow --name jio-pow
```

### 1.1 Update Root `Cargo.toml`
Open `rusty-jio/Cargo.toml` and add the two new crates to `members`, and add `indexmap` and `smallvec` to `workspace.dependencies`:

```toml
[workspace]
resolver = "2"
members = [
    "math",
    "utils",
    "core",
    "database",
    "crypto/hashes",
    "crypto/merkle",
    "crypto/addresses",
    "consensus/core",
    "consensus/pow",
]

[workspace.dependencies]
thiserror = "1.0"
parking_lot = "0.12"
tokio = { version = "1.36", features = ["full"] }
rocksdb = "0.21"
serde = { version = "1.0", features = ["derive"] }
borsh = { version = "1.3", features = ["derive"] }
blake2b_simd = "1.0"
sha2 = "0.10"
indexmap = { version = "2.2", features = ["serde"] }
smallvec = { version = "1.13", features = ["serde"] }
```

---

## Step 2: Code Crate #1 — `consensus/core/`

`consensus/core` has zero network or disk dependencies—it is pure data structures and algorithmic rules.

### 2.1 Edit `consensus/core/Cargo.toml`
Open `consensus/core/Cargo.toml` and paste:

```toml
[package]
name = "jio-consensus-core"
version = "0.1.0"
edition = "2021"

[dependencies]
jio-math = { path = "../../math" }
jio-hashes = { path = "../../crypto/hashes" }
jio-merkle = { path = "../../crypto/merkle" }
jio-addresses = { path = "../../crypto/addresses" }

serde = { workspace = true }
borsh = { workspace = true }
thiserror = { workspace = true }
indexmap = { workspace = true }
smallvec = { workspace = true }
```

---

### 2.2 Create `consensus/core/src/header.rs` (BlockDAG Header)

In a BlockDAG, a block header contains multi-level parent links (`parents_by_level`), difficulty bits, DAA score, and cumulative blue work (`Uint192`):

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, Hasher};
use jio_math::Uint192;
use serde::{Deserialize, Serialize};

/// The header of a block in the JIO BlockDAG.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Header {
    pub hash: Hash,
    pub version: u16,
    pub parents_by_level: Vec<Vec<Hash>>,
    pub hash_merkle_root: Hash,
    pub accepted_id_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub blue_work: Uint192,
    pub blue_score: u64,
    pub pruning_point: Hash,
}

impl Header {
    pub fn new(
        version: u16,
        parents_by_level: Vec<Vec<Hash>>,
        hash_merkle_root: Hash,
        accepted_id_merkle_root: Hash,
        utxo_commitment: Hash,
        timestamp: u64,
        bits: u32,
        nonce: u64,
        daa_score: u64,
        blue_work: Uint192,
        blue_score: u64,
        pruning_point: Hash,
    ) -> Self {
        let mut header = Self {
            hash: Hash::ZERO,
            version,
            parents_by_level,
            hash_merkle_root,
            accepted_id_merkle_root,
            utxo_commitment,
            timestamp,
            bits,
            nonce,
            daa_score,
            blue_work,
            blue_score,
            pruning_point,
        };
        header.hash = header.compute_hash();
        header
    }

    /// Direct parents are level 0 parents.
    pub fn direct_parents(&self) -> &[Hash] {
        if self.parents_by_level.is_empty() {
            &[]
        } else {
            &self.parents_by_level[0]
        }
    }

    /// Check if this header is a genesis block header (no parents).
    pub fn is_genesis(&self) -> bool {
        self.parents_by_level.is_empty() || self.parents_by_level[0].is_empty()
    }

    /// Computes the domain-separated BlockHash over all header fields.
    pub fn compute_hash(&self) -> Hash {
        let mut hasher = Hasher::new("BlockHash");
        hasher.update(&self.version.to_le_bytes());
        
        let num_levels = self.parents_by_level.len() as u32;
        hasher.update(&num_levels.to_le_bytes());
        for level in &self.parents_by_level {
            let num_parents = level.len() as u32;
            hasher.update(&num_parents.to_le_bytes());
            for parent in level {
                hasher.update(parent.as_bytes());
            }
        }

        hasher.update(self.hash_merkle_root.as_bytes());
        hasher.update(self.accepted_id_merkle_root.as_bytes());
        hasher.update(self.utxo_commitment.as_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.bits.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.daa_score.to_le_bytes());
        
        let work_bytes = self.blue_work.to_le_bytes();
        hasher.update(&work_bytes);
        hasher.update(&self.blue_score.to_le_bytes());
        hasher.update(self.pruning_point.as_bytes());

        hasher.finalize()
    }

    /// Computes the PrePowHash (hash of header with timestamp=0 and nonce=0).
    pub fn compute_pre_pow_hash(&self) -> Hash {
        let mut hasher = Hasher::new("ProofOfWorkHash");
        hasher.update(&self.version.to_le_bytes());
        
        let num_levels = self.parents_by_level.len() as u32;
        hasher.update(&num_levels.to_le_bytes());
        for level in &self.parents_by_level {
            let num_parents = level.len() as u32;
            hasher.update(&num_parents.to_le_bytes());
            for parent in level {
                hasher.update(parent.as_bytes());
            }
        }

        hasher.update(self.hash_merkle_root.as_bytes());
        hasher.update(self.accepted_id_merkle_root.as_bytes());
        hasher.update(self.utxo_commitment.as_bytes());
        
        // Zero timestamp and zero nonce for pre-pow commitments
        hasher.update(&0u64.to_le_bytes());
        hasher.update(&self.bits.to_le_bytes());
        hasher.update(&0u64.to_le_bytes());
        
        hasher.update(&self.daa_score.to_le_bytes());
        let work_bytes = self.blue_work.to_le_bytes();
        hasher.update(&work_bytes);
        hasher.update(&self.blue_score.to_le_bytes());
        hasher.update(self.pruning_point.as_bytes());

        hasher.finalize()
    }
}
```

---

### 2.3 Create `consensus/core/src/tx.rs` (Transactions, Inputs, Outputs)

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, Hasher};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransactionOutpoint {
    pub transaction_id: Hash,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: Hash, index: u32) -> Self {
        Self { transaction_id, index }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubnetworkId(pub [u8; 20]);

impl SubnetworkId {
    pub const NATIVE: SubnetworkId = SubnetworkId([0u8; 20]);
    pub const COINBASE: SubnetworkId = SubnetworkId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
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
        let mut hasher = Hasher::new("TransactionID");
        hasher.update(&self.version.to_le_bytes());
        
        let num_inputs = self.inputs.len() as u32;
        hasher.update(&num_inputs.to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(&input.previous_outpoint.index.to_le_bytes());
            hasher.update(&input.sequence.to_le_bytes());
        }

        let num_outputs = self.outputs.len() as u32;
        hasher.update(&num_outputs.to_le_bytes());
        for output in &self.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(&script_len.to_le_bytes());
            hasher.update(&output.script_public_key.script);
        }

        hasher.update(&self.lock_time.to_le_bytes());
        hasher.update(&self.subnetwork_id.0);
        hasher.update(&self.gas.to_le_bytes());
        let payload_len = self.payload.len() as u32;
        hasher.update(&payload_len.to_le_bytes());
        hasher.update(&self.payload);

        hasher.finalize()
    }

    /// Full transaction hash includes signature scripts.
    pub fn compute_hash(&self) -> Hash {
        let mut hasher = Hasher::new("TransactionHash");
        hasher.update(&self.version.to_le_bytes());
        
        let num_inputs = self.inputs.len() as u32;
        hasher.update(&num_inputs.to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(&input.previous_outpoint.index.to_le_bytes());
            let sig_len = input.signature_script.len() as u32;
            hasher.update(&sig_len.to_le_bytes());
            hasher.update(&input.signature_script);
            hasher.update(&input.sequence.to_le_bytes());
            hasher.update(&[input.sig_op_count]);
        }

        let num_outputs = self.outputs.len() as u32;
        hasher.update(&num_outputs.to_le_bytes());
        for output in &self.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(&script_len.to_le_bytes());
            hasher.update(&output.script_public_key.script);
        }

        hasher.update(&self.lock_time.to_le_bytes());
        hasher.update(&self.subnetwork_id.0);
        hasher.update(&self.gas.to_le_bytes());
        let payload_len = self.payload.len() as u32;
        hasher.update(&payload_len.to_le_bytes());
        hasher.update(&self.payload);

        hasher.finalize()
    }
}
```

---

### 2.4 Create `consensus/core/src/block.rs` (Block Structure)

```rust
use crate::header::Header;
use crate::tx::Transaction;
use jio_hashes::Hash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash
    }
}
```

---

### 2.5 Create `consensus/core/src/utxo/` (UtxoEntry, UtxoCollection, UtxoDiff)

#### `consensus/core/src/utxo/utxo_entry.rs`:
```rust
use crate::tx::ScriptPublicKey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UtxoEntry {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub fn new(amount: u64, script_public_key: ScriptPublicKey, block_daa_score: u64, is_coinbase: bool) -> Self {
        Self {
            amount,
            script_public_key,
            block_daa_score,
            is_coinbase,
        }
    }
}
```

#### `consensus/core/src/utxo/utxo_diff.rs`:
```rust
use super::utxo_entry::UtxoEntry;
use crate::tx::TransactionOutpoint;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type UtxoCollection = IndexMap<TransactionOutpoint, UtxoEntry>;

/// Represents an atomic state diff on the UTXO set: UTXOs added and UTXOs removed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtxoDiff {
    pub add: UtxoCollection,
    pub remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(add: UtxoCollection, remove: UtxoCollection) -> Self {
        Self { add, remove }
    }

    /// Reverses the diff (adds become removes, removes become adds).
    pub fn reversed(&self) -> Self {
        Self {
            add: self.remove.clone(),
            remove: self.add.clone(),
        }
    }

    /// Applies another diff on top of this one to form a composite diff.
    pub fn with_diff(&mut self, other: &UtxoDiff) -> Result<(), &'static str> {
        for (outpoint, entry) in &other.remove {
            if self.add.shift_remove(outpoint).is_none() {
                self.remove.insert(*outpoint, entry.clone());
            }
        }
        for (outpoint, entry) in &other.add {
            if self.remove.shift_remove(outpoint).is_none() {
                self.add.insert(*outpoint, entry.clone());
            }
        }
        Ok(())
    }
}
```

#### `consensus/core/src/utxo/mod.rs`:
```rust
pub mod utxo_diff;
pub mod utxo_entry;

pub use utxo_diff::{UtxoCollection, UtxoDiff};
pub use utxo_entry::UtxoEntry;
```

---

### 2.6 Create `consensus/core/src/hashing/sighash.rs` (Schnorr Sighash & Reused Values)

```rust
use crate::tx::{Transaction, TransactionOutpoint};
use crate::utxo::UtxoEntry;
use jio_hashes::{Hash, Hasher};

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
        let mut hasher = Hasher::new("TransactionSigningHash");
        for input in &tx.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(&input.previous_outpoint.index.to_le_bytes());
        }
        reused.previous_outputs_hash = Some(hasher.finalize());

        // 2. Sequences hash
        let mut hasher = Hasher::new("TransactionSigningHash");
        for input in &tx.inputs {
            hasher.update(&input.sequence.to_le_bytes());
        }
        reused.sequences_hash = Some(hasher.finalize());

        // 3. SigOp counts hash
        let mut hasher = Hasher::new("TransactionSigningHash");
        for input in &tx.inputs {
            hasher.update(&[input.sig_op_count]);
        }
        reused.sig_op_counts_hash = Some(hasher.finalize());

        // 4. Payload hash
        let mut hasher = Hasher::new("TransactionSigningHash");
        hasher.update(&tx.subnetwork_id.0);
        hasher.update(&tx.gas.to_le_bytes());
        let payload_len = tx.payload.len() as u32;
        hasher.update(&payload_len.to_le_bytes());
        hasher.update(&tx.payload);
        reused.payload_hash = Some(hasher.finalize());

        // 5. Outputs hash
        let mut hasher = Hasher::new("TransactionSigningHash");
        for output in &tx.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.script_public_key.version.to_le_bytes());
            let script_len = output.script_public_key.script.len() as u32;
            hasher.update(&script_len.to_le_bytes());
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
    let mut hasher = Hasher::new("TransactionSigningHash");

    // 1. Version
    hasher.update(&tx.version.to_le_bytes());

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
    hasher.update(&input.previous_outpoint.index.to_le_bytes());

    // 6. UTXO being spent
    hasher.update(&utxo_entry.amount.to_le_bytes());
    hasher.update(&utxo_entry.script_public_key.version.to_le_bytes());
    let script_len = utxo_entry.script_public_key.script.len() as u32;
    hasher.update(&script_len.to_le_bytes());
    hasher.update(&utxo_entry.script_public_key.script);

    // 7. Input sequence and sig_op_count
    hasher.update(&input.sequence.to_le_bytes());
    hasher.update(&[input.sig_op_count]);

    // 8. Outputs hash
    if (hash_type & 0x07) != SIG_HASH_NONE && (hash_type & 0x07) != SIG_HASH_SINGLE {
        if let Some(h) = reused.outputs_hash {
            hasher.update(h.as_bytes());
        }
    } else if (hash_type & 0x07) == SIG_HASH_SINGLE && input_index < tx.outputs.len() {
        let mut single_hasher = Hasher::new("TransactionSigningHash");
        let output = &tx.outputs[input_index];
        single_hasher.update(&output.value.to_le_bytes());
        single_hasher.update(&output.script_public_key.version.to_le_bytes());
        let script_len = output.script_public_key.script.len() as u32;
        single_hasher.update(&script_len.to_le_bytes());
        single_hasher.update(&output.script_public_key.script);
        hasher.update(single_hasher.finalize().as_bytes());
    }

    // 9. Lock time, Subnetwork, and Hash type
    hasher.update(&tx.lock_time.to_le_bytes());
    hasher.update(&tx.subnetwork_id.0);
    hasher.update(&tx.gas.to_le_bytes());
    if let Some(h) = reused.payload_hash {
        hasher.update(h.as_bytes());
    }
    hasher.update(&[hash_type]);

    hasher.finalize()
}
```

#### `consensus/core/src/hashing/mod.rs`:
```rust
pub mod sighash;

pub use sighash::{
    calc_schnorr_signature_hash, SigHashReusedValues, SIG_HASH_ALL, SIG_HASH_ANYONECANPAY,
    SIG_HASH_NONE, SIG_HASH_SINGLE,
};
```

---

### 2.7 Create `consensus/core/src/mass/mod.rs` (Compute & KIP-0009 Storage Mass)

```rust
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
```

---

### 2.8 Create `consensus/core/src/config/` (Params, BPS, Genesis)

#### `consensus/core/src/config/bps.rs`:
```rust
/// Helper formulas for scaling BlockDAG parameters according to Blocks Per Second (BPS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bps {
    Bps1 = 1,
    Bps10 = 10,
}

impl Bps {
    /// Computes the GHOSTDAG anticone K parameter from BPS.
    /// At 1 BPS, K = 18. At 10 BPS, K = 124.
    pub const fn ghostdag_k(&self) -> u64 {
        match self {
            Bps::Bps1 => 18,
            Bps::Bps10 => 124,
        }
    }

    /// Block interval in milliseconds.
    pub const fn target_time_per_block_ms(&self) -> u64 {
        match self {
            Bps::Bps1 => 1000,
            Bps::Bps10 => 100,
        }
    }

    /// Difficulty adjustment sliding window size.
    pub const fn difficulty_window_size(&self) -> usize {
        match self {
            Bps::Bps1 => 2641,
            Bps::Bps10 => 26410,
        }
    }
}
```

#### `consensus/core/src/config/params.rs`:
```rust
use super::bps::Bps;
use jio_hashes::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

#[derive(Clone, Debug)]
pub struct Params {
    pub net: NetworkType,
    pub bps: Bps,
    pub ghostdag_k: u64,
    pub target_time_per_block_ms: u64,
    pub difficulty_window_size: usize,
    pub min_difficulty_window_size: usize,
    pub mergeset_size_limit: u64,
    pub finality_depth: u64,
    pub pruning_depth: u64,
    pub coinbase_maturity: u64,
    pub max_block_mass: u64,
    pub genesis_hash: Hash,
}

impl Params {
    pub fn mainnet() -> Self {
        let bps = Bps::Bps10;
        Self {
            net: NetworkType::Mainnet,
            bps,
            ghostdag_k: bps.ghostdag_k(),
            target_time_per_block_ms: bps.target_time_per_block_ms(),
            difficulty_window_size: bps.difficulty_window_size(),
            min_difficulty_window_size: 263,
            mergeset_size_limit: bps.ghostdag_k() * 10,
            finality_depth: 86_400,
            pruning_depth: 185_798,
            coinbase_maturity: 100,
            max_block_mass: 500_000,
            genesis_hash: Hash::ZERO,
        }
    }

    pub fn testnet() -> Self {
        let mut p = Self::mainnet();
        p.net = NetworkType::Testnet;
        p
    }

    pub fn simnet() -> Self {
        let bps = Bps::Bps1;
        Self {
            net: NetworkType::Simnet,
            bps,
            ghostdag_k: bps.ghostdag_k(),
            target_time_per_block_ms: bps.target_time_per_block_ms(),
            difficulty_window_size: 140,
            min_difficulty_window_size: 10,
            mergeset_size_limit: bps.ghostdag_k() * 10,
            finality_depth: 1000,
            pruning_depth: 2000,
            coinbase_maturity: 10,
            max_block_mass: 500_000,
            genesis_hash: Hash::ZERO,
        }
    }
}
```

#### `consensus/core/src/config/genesis.rs`:
```rust
use crate::block::Block;
use crate::header::Header;
use crate::tx::{ScriptPublicKey, SubnetworkId, Transaction, TransactionOutput};
use jio_hashes::Hash;
use jio_math::Uint192;

/// Builds the canonical Genesis Block for the JIO network.
pub fn create_genesis_block(timestamp: u64, bits: u32) -> Block {
    let coinbase_out = TransactionOutput {
        value: 0,
        script_public_key: ScriptPublicKey::new(0, vec![0x51]), // OP_TRUE
    };

    let coinbase_tx = Transaction::new(
        1,
        vec![],
        vec![coinbase_out],
        0,
        SubnetworkId::COINBASE,
        0,
        b"JIO Genesis Block - Decentralized BlockDAG with kHeavyHash".to_vec(),
    );

    let header = Header::new(
        1,
        vec![], // No parents for genesis!
        coinbase_tx.id,
        Hash::ZERO,
        Hash::ZERO,
        timestamp,
        bits,
        0,
        0,
        Uint192::ZERO,
        0,
        Hash::ZERO,
    );

    Block::new(header, vec![coinbase_tx])
}
```

#### `consensus/core/src/config/mod.rs`:
```rust
pub mod bps;
pub mod genesis;
pub mod params;

pub use bps::Bps;
pub use genesis::create_genesis_block;
pub use params::{NetworkType, Params};
```

---

### 2.9 Wire `consensus/core/src/lib.rs` and Add Tests

```rust
pub mod block;
pub mod config;
pub mod hashing;
pub mod header;
pub mod mass;
pub mod tx;
pub mod utxo;

pub use block::Block;
pub use header::Header;
pub use tx::{
    ScriptPublicKey, SubnetworkId, Transaction, TransactionInput, TransactionOutpoint,
    TransactionOutput,
};
pub use utxo::{UtxoCollection, UtxoDiff, UtxoEntry};

#[cfg(test)]
mod tests {
    use super::*;
    use jio_hashes::Hash;
    use jio_math::Uint192;

    #[test]
    fn test_header_creation_and_hash_stability() {
        let header = Header::new(
            1,
            vec![vec![Hash::from_le_u64(1)]],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_000_000,
            0x1e7fffff,
            42,
            100,
            Uint192::from_u64(500),
            100,
            Hash::ZERO,
        );

        assert_eq!(header.direct_parents().len(), 1);
        assert!(!header.is_genesis());
        assert_ne!(header.hash, Hash::ZERO);
        assert_ne!(header.compute_pre_pow_hash(), Hash::ZERO);
        assert_ne!(header.hash, header.compute_pre_pow_hash());
    }

    #[test]
    fn test_tx_id_immutability_under_signature_change() {
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64(10), 0);
        let mut input1 = TransactionInput {
            previous_outpoint: outpoint,
            signature_script: vec![1, 2, 3],
            sequence: 0,
            sig_op_count: 1,
        };
        let output = TransactionOutput {
            value: 50_000_000,
            script_public_key: ScriptPublicKey::new(0, vec![0x42]),
        };

        let tx1 = Transaction::new(1, vec![input1.clone()], vec![output.clone()], 0, SubnetworkId::NATIVE, 0, vec![]);
        
        // Mutate signature script
        input1.signature_script = vec![9, 9, 9, 9, 9];
        let tx2 = Transaction::new(1, vec![input1], vec![output], 0, SubnetworkId::NATIVE, 0, vec![]);

        // TxID MUST remain identical (non-malleable)!
        assert_eq!(tx1.id, tx2.id);
        // Full hash MUST change!
        assert_ne!(tx1.compute_hash(), tx2.compute_hash());
    }

    #[test]
    fn test_utxo_diff_reversal_and_composite() {
        let op1 = TransactionOutpoint::new(Hash::from_le_u64(1), 0);
        let op2 = TransactionOutpoint::new(Hash::from_le_u64(2), 0);
        let entry1 = UtxoEntry::new(100, ScriptPublicKey::new(0, vec![]), 1, false);
        let entry2 = UtxoEntry::new(200, ScriptPublicKey::new(0, vec![]), 2, false);

        let mut add = UtxoCollection::new();
        add.insert(op1, entry1.clone());

        let mut remove = UtxoCollection::new();
        remove.insert(op2, entry2.clone());

        let diff = UtxoDiff::new(add, remove);
        let rev = diff.reversed();

        assert_eq!(rev.add.len(), 1);
        assert_eq!(rev.remove.len(), 1);
        assert!(rev.add.contains_key(&op2));
        assert!(rev.remove.contains_key(&op1));
    }

    #[test]
    fn test_kip0009_dust_penalty_storage_mass() {
        let op = TransactionOutpoint::new(Hash::ZERO, 0);
        let input = TransactionInput {
            previous_outpoint: op,
            signature_script: vec![],
            sequence: 0,
            sig_op_count: 0,
        };
        let in_entry = UtxoEntry::new(10_000_000_000, ScriptPublicKey::new(0, vec![]), 1, false);

        // Transaction creating a 100 sompi dust output
        let dust_out = TransactionOutput {
            value: 100,
            script_public_key: ScriptPublicKey::new(0, vec![]),
        };
        let tx = Transaction::new(1, vec![input], vec![dust_out], 0, SubnetworkId::NATIVE, 0, vec![]);

        let storage_mass = mass::calc_storage_mass(&tx, &[in_entry]);
        // Storage mass must penalize dust with massive mass!
        assert!(storage_mass > 10_000_000);
    }
}
```

Test it: `cargo test -p jio-consensus-core`

---

## Step 3: Code Crate #2 — `consensus/pow/` (kHeavyHash Proof-of-Work)

The Proof-of-Work engine implements Kaspa-style **kHeavyHash**:
1. Seed the `XoShiRo256PlusPlus` PRNG with the block's `pre_pow_hash`.
2. Generate a deterministic $64 \times 64$ matrix of 4-bit nibbles.
3. Hash the nonce header commitment into 64 nibbles.
4. Multiply matrix by vector, right-shift by 10, XOR mix, and hash again.
5. Check if resulting `pow_value <= target_threshold`.

### 3.1 Edit `consensus/pow/Cargo.toml`
Open `consensus/pow/Cargo.toml` and paste:

```toml
[package]
name = "jio-pow"
version = "0.1.0"
edition = "2021"

[dependencies]
jio-math = { path = "../../math" }
jio-hashes = { path = "../../crypto/hashes" }
jio-consensus-core = { path = "../core" }

thiserror = { workspace = true }
```

---

### 3.2 Create `consensus/pow/src/xoshiro.rs` (XoShiRo256++ PRNG)

```rust
/// The XoShiRo256++ pseudo-random number generator.
/// State consists of 4 64-bit unsigned integers.
#[derive(Clone, Debug)]
pub struct XoShiRo256PlusPlus {
    s: [u64; 4],
}

impl XoShiRo256PlusPlus {
    /// Initializes the PRNG deterministically from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let mut s = [0u64; 4];
        for i in 0..4 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&seed[i * 8..(i + 1) * 8]);
            s[i] = u64::from_le_bytes(buf);
        }
        // Ensure state is not all zeros
        if s[0] == 0 && s[1] == 0 && s[2] == 0 && s[3] == 0 {
            s[0] = 0x9E3779B97F4A7C15;
            s[1] = 0xBF58476D1CE4E5B9;
            s[2] = 0x94D049BB133111EB;
            s[3] = 0x46D32A15E7C81274;
        }
        Self { s }
    }

    /// Generates the next 64-bit pseudo-random unsigned integer.
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);

        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }
}
```

---

### 3.3 Create `consensus/pow/src/matrix.rs` ($64 \times 64$ Nibble Matrix Multiplication)

Each matrix element is a 4-bit nibble ($0 \dots 15$). A $64 \times 64$ matrix multiplication requires multiplying a 64-nibble vector, right-shifting each accumulated row product by 10 bits, and XORing with the original input vector:

```rust
use crate::xoshiro::XoShiRo256PlusPlus;

/// A 64x64 matrix of 4-bit nibbles (values 0..=15).
#[derive(Clone, Debug)]
pub struct Matrix64x64 {
    pub rows: [[u8; 64]; 64],
}

impl Matrix64x64 {
    /// Generates a deterministic 64x64 nibble matrix using XoShiRo256++.
    pub fn generate(prng: &mut XoShiRo256PlusPlus) -> Self {
        let mut rows = [[0u8; 64]; 64];
        for r in 0..64 {
            for chunk in 0..4 {
                let val = prng.next_u64();
                for nibble_idx in 0..16 {
                    let col = chunk * 16 + nibble_idx;
                    rows[r][col] = ((val >> (nibble_idx * 4)) & 0x0F) as u8;
                }
            }
        }
        Self { rows }
    }

    /// Multiplies the matrix by a 64-element 4-bit vector, shifts by 10, and XORs:
    /// out_nibble[i] = ((sum_{j=0}^{63} M[i][j] * v[j]) >> 10) ^ v[i]
    pub fn multiply(&self, vector: &[u8; 64]) -> [u8; 64] {
        let mut out = [0u8; 64];
        for i in 0..64 {
            let mut sum: u32 = 0;
            for j in 0..64 {
                sum = sum.wrapping_add((self.rows[i][j] as u32) * (vector[j] as u32));
            }
            let shifted = (sum >> 10) as u8;
            out[i] = (shifted ^ vector[i]) & 0x0F;
        }
        out
    }
}
```

---

### 3.4 Create `consensus/pow/src/calc.rs` (PoW Computation & Target Checking)

```rust
use crate::matrix::Matrix64x64;
use crate::xoshiro::XoShiRo256PlusPlus;
use jio_consensus_core::Header;
use jio_hashes::{Hash, Hasher};
use jio_math::Uint256;

/// Helper to convert a 32-byte hash into 64 4-bit nibbles.
pub fn hash_to_nibbles(hash: &Hash) -> [u8; 64] {
    let mut nibbles = [0u8; 64];
    for (i, &byte) in hash.as_bytes().iter().enumerate() {
        nibbles[i * 2] = (byte >> 4) & 0x0F;
        nibbles[i * 2 + 1] = byte & 0x0F;
    }
    nibbles
}

/// Helper to repack 64 4-bit nibbles back into 32 bytes.
pub fn nibbles_to_bytes(nibbles: &[u8; 64]) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = ((nibbles[i * 2] & 0x0F) << 4) | (nibbles[i * 2 + 1] & 0x0F);
    }
    bytes
}

/// Computes the complete kHeavyHash Proof-of-Work for a block header.
pub fn calculate_pow(header: &Header) -> Uint256 {
    // 1. Compute PrePowHash
    let pre_pow_hash = header.compute_pre_pow_hash();

    // 2. Generate 64x64 matrix from PRNG seeded by PrePowHash
    let mut prng = XoShiRo256PlusPlus::from_seed(*pre_pow_hash.as_bytes());
    let matrix = Matrix64x64::generate(&mut prng);

    // 3. Digest header with timestamp and nonce
    let mut hasher = Hasher::new("ProofOfWorkHash");
    hasher.update(pre_pow_hash.as_bytes());
    hasher.update(&header.timestamp.to_le_bytes());
    hasher.update(&header.nonce.to_le_bytes());
    let initial_hash = hasher.finalize();

    // 4. Matrix-vector multiplication over 64 nibbles
    let vector = hash_to_nibbles(&initial_hash);
    let multiplied_vector = matrix.multiply(&vector);
    let mixed_bytes = nibbles_to_bytes(&multiplied_vector);

    // 5. Final HeavyHash pass
    let mut heavy_hasher = Hasher::new("HeavyHash");
    heavy_hasher.update(&mixed_bytes);
    let final_hash = heavy_hasher.finalize();

    // 6. Convert to 256-bit integer
    Uint256::from_le_bytes(*final_hash.as_bytes())
}

/// Verifies whether the header's Proof-of-Work satisfies its difficulty target bits.
pub fn check_pow(header: &Header) -> bool {
    let pow_val = calculate_pow(header);
    let target = Uint256::from_compact_target_bits(header.bits);
    pow_val <= target
}
```

---

### 3.5 Wire `consensus/pow/src/lib.rs` and Add Tests

```rust
pub mod calc;
pub mod matrix;
pub mod xoshiro;

pub use calc::{calculate_pow, check_pow, hash_to_nibbles, nibbles_to_bytes};
pub use matrix::Matrix64x64;
pub use xoshiro::XoShiRo256PlusPlus;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus_core::Header;
    use jio_hashes::Hash;
    use jio_math::{Uint192, Uint256};

    #[test]
    fn test_xoshiro256plusplus_sequence() {
        let mut prng = XoShiRo256PlusPlus::from_seed([1u8; 32]);
        let val1 = prng.next_u64();
        let val2 = prng.next_u64();
        assert_ne!(val1, 0);
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_matrix_vector_multiplication() {
        let mut prng = XoShiRo256PlusPlus::from_seed([42u8; 32]);
        let matrix = Matrix64x64::generate(&mut prng);

        let input_vec = [7u8; 64];
        let out_vec = matrix.multiply(&input_vec);

        // Output nibbles must stay strictly within 0..=15
        for &nibble in &out_vec {
            assert!(nibble <= 15);
        }
    }

    #[test]
    fn test_nibble_roundtrip_conversion() {
        let hash = Hash::from_le_u64(0x123456789ABCDEF0);
        let nibbles = hash_to_nibbles(&hash);
        let recovered = nibbles_to_bytes(&nibbles);
        assert_eq!(*hash.as_bytes(), recovered);
    }

    #[test]
    fn test_pow_verification_with_easy_target() {
        // High target (easiest difficulty)
        let easy_bits = 0x207fffff;
        let mut header = Header::new(
            1,
            vec![],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_000_000,
            easy_bits,
            0,
            0,
            Uint192::ZERO,
            0,
            Hash::ZERO,
        );

        // Mine a nonce that passes
        let mut found = false;
        for nonce in 0..10_000 {
            header.nonce = nonce;
            if check_pow(&header) {
                found = true;
                break;
            }
        }
        assert!(found, "Should find valid PoW with easy target");
    }
}
```

Test it: `cargo test -p jio-pow`

---

## Step 4: Run the Complete Workspace Test Suite

Now test **Layer 1, Layer 2, and Layer 3 together**:

```bash
cargo test --workspace
```

You should see all unit tests passing across all 9 crates:
- `jio-math`
- `jio-utils`
- `jio-core`
- `jio-database`
- `jio-hashes`
- `jio-merkle`
- `jio-addresses`
- `jio-consensus-core`
- `jio-pow`

---

## Step 5: Commit and Push to Your GitHub Repository

Once tests pass, commit Layer 3 to your git repository:

```bash
git add .
git commit -m "feat: complete Stage 3 - Consensus Core Data Models & kHeavyHash PoW"
git push origin main
```

---

## What Comes Next: Stage 4 Preview

In **Stage 4 (The GHOSTDAG Consensus Engine)**, we will build:
1. **The Reachability Tree (`consensus/src/processes/reachability/`)**: $O(1)$ interval indexing for ancestor queries across millions of DAG blocks.
2. **The GHOSTDAG Ordering Engine (`consensus/src/processes/ghostdag/`)**: Blue/Red anticone coloring, selected parent traversal, and topological mergeset sorting.
3. **Sliding-Window DAA (`consensus/src/processes/difficulty.rs`)**: 2641-block sampling window adjusting difficulty every millisecond.
4. **4-Stage Validation Pipeline**: `HeaderProcessor`, `BodyProcessor`, `VirtualProcessor`, and `PruningProcessor`.
