---
title: rusty-jiopad Phase 1 File-wise Logic & Implementation Plan
aliases: [Phase 1 plan, Foundations plan, L0-L2 plan]
tags: [project-plan, phase1, rust, architecture, logic-spec]
project: "[[rusty-jiopad]]"
parent: "[[rusty-jiopad-module-plan]]"
status: active
created: 2026-10-02
---

# Phase 1: Foundations, Cryptography, Async Runtime & Consensus Core

Phase 1 establishes the bedrock layers (**L0, L1, and L2**) of the **`rusty-jio`** node. All higher subsystems (Database, Consensus Pipeline, Mempool, P2P Protocol, RPC, and Daemon) depend directly on the types, math, cryptography, runtime traits, and consensus interfaces implemented in this phase.

---

## 1. Phase 1 Scope & Dependency Structure

```
Layer 0: Foundational Math, Utilities & Metrics
   ├── math              (U192, U256, U512 big numbers, difficulty conversions)
   ├── utils             (Hex, triggers, memory size, networking, atomics)
   └── metrics/core      (Metric data structures & snapshots)
          │
Layer 1: Cryptographic Primitives
   └── crypto
       ├── hashes        (BLAKE3 domain hashers, cSHAKE, Keccak, pow_hashers)
       ├── addresses     (Bech32 address encoding/decoding: jio, jiotest, jiosim, jiodev)
       ├── merkle        (Merkle root calculation for transactions)
       ├── muhash        (3072-bit multi-set additive hash for UTXO commitments)
       └── txscript      (Script engine, opcodes, stack machine, standard scripts)
          │
Layer 2: Async Framework & Canonical Consensus Domain
   ├── core              (AsyncService, Core supervisor, Tokio runtime, signals, logging)
   └── consensus/core    (Block, Header, Tx, Params, Genesis, UtxoDiff, ConsensusApi)
```

---

## 2. Workspace Root Files

### `Cargo.toml`
* **Path**: [`Cargo.toml`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/Cargo.toml)
* **Purpose**: Root workspace manifest defining members, shared dependency versions, and compiler optimization flags.
* **Logic & Implementation**:
  - `[workspace]` table declaring all crates:
    `members = ["core", "math", "utils", "metrics/core", "crypto/hashes", "crypto/addresses", "crypto/merkle", "crypto/muhash", "crypto/txscript", "consensus/core", ...]`
  - `[workspace.package]`: `edition = "2024"`, `rust-version = "1.97.0"`, `license = "ISC"`.
  - `[workspace.dependencies]`: Centralize external dependency versions to avoid version mismatches across crates:
    - `blake3 = "=1.5.4"`
    - `tokio = { version = "1.39", features = ["full"] }`
    - `parking_lot = "0.12"`
    - `thiserror = "1.0"`
    - `serde = { version = "1.0", features = ["derive"] }`
    - `borsh = { version = "1.5", features = ["derive"] }`
    - `secp256k1 = { version = "0.29", features = ["global-context", "rand-std"] }`
  - `[profile.release]`: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`.
  - `[profile.dev]`: `opt-level = 1` for fast iteration during unit tests.

### `.rustfmt.toml`
* **Path**: [`.rustfmt.toml`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/.rustfmt.toml)
* **Logic**: Enforce consistent formatting across the team: `max_width = 120`, `tab_spaces = 4`, `edition = "2024"`, `use_small_heuristics = "Max"`.

### `clippy.toml`
* **Path**: [`clippy.toml`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/clippy.toml)
* **Logic**: Configure linter thresholds (`too-many-arguments-threshold = 8`, `type-complexity-threshold = 250`).

### `check.ps1` / `check`
* **Path**: [`check.ps1`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/check.ps1) & [`check`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/check)
* **Logic**: PowerShell & Unix bash scripts running:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --workspace --all-targets -- -D warnings`
  3. `cargo test --workspace`

---

## 3. Layer 0: Math, Utilities & Metrics

### 3.1 `math` Crate
Located at `math/`. Crate name: `jio-math`.

#### [`math/Cargo.toml`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/math/Cargo.toml)
- Dependencies: `borsh`, `serde`, `wasm-bindgen` (optional).

#### [`math/src/lib.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/math/src/lib.rs)
- Re-exports `uint::{U192, U256, U512}`, `int::*`.

#### [`math/src/uint.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/math/src/uint.rs)
- **Types**:
  - `U192([u64; 3])`: 192-bit unsigned integer (used for intermediate difficulty math).
  - `U256([u64; 4])`: 256-bit unsigned integer (used for block targets, PoW hashes, and blue work accumulation).
  - `U512([u64; 8])`: 512-bit unsigned integer (used to prevent overflow during target $\times$ window multiplications).
- **Core Logic & Methods**:
  - Macro `construct_uint!(struct $name($n_words))` generating:
    - `Zero::zero()`, `One::one()`, `MAX`.
    - Addition/Subtraction with overflow detection: `overflowing_add()`, `saturating_add()`, `overflowing_sub()`.
    - Long multiplication (schoolbook or Karatsuba algorithm) with carry propagation across 64-bit limbs.
    - Full division and remainder: `div_mod(dividend, divisor) -> (quotient, remainder)`.
    - Bitwise shifts: `shl()`, `shr()`, `leading_zeros()`, `trailing_zeros()`, `bits()`.
    - Conversion methods: `to_le_bytes()`, `from_le_bytes()`, `to_be_bytes()`, `from_be_bytes()`.
    - Target conversion: `from_compact_target_bits(bits: u32) -> (U256, bool)` (unpacking 8-bit exponent and 24-bit mantissa) and `to_compact_target_bits(target: U256) -> u32`.
- **Invariants & Edge Cases**:
  - Division by zero must panic or return `Err(MathError::DivisionByZero)`.
  - Negative compact targets (sign bit set in mantissa) must be rejected.
  - If compact target exponent exceeds 32, it represents an overflow target.
- **Testing**:
  - Property tests (`proptest`): Commutativity ($a + b == b + a$), associativity ($a \times (b \times c) == (a \times b) \times c$), and division identity ($(a / b) \times b + (a \% b) == a$).

#### [`math/src/int.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/math/src/int.rs)
- Signed integer utilities for DAA window drift calculations (difference between expected block times and actual timestamps).

#### [`math/src/wasm.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/math/src/wasm.rs)
- JavaScript bindings for `U256` serialization to/from hex string and `BigInt`.

---

### 3.2 `utils` Crate
Located at `utils/`. Crate name: `jio-utils`.

#### [`utils/src/hex.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/hex.rs)
- **Traits**:
  - `ToHex`: `fn to_hex(&self) -> String` and `fn write_hex(&self, f: &mut Formatter)`.
  - `FromHex`: `fn from_hex(hex_str: &str) -> Result<Self, FromHexError>`.
- **Logic**: Fast SIMD-accelerated or lookup-table hex encoder/decoder avoiding heap allocation for fixed-length byte arrays (`[u8; 32]`).

#### [`utils/src/mem_size.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/mem_size.rs)
- **Trait**: `MemSizeEstimator { fn estimate_mem_size(&self) -> usize; }`
- **Logic**: Calculates deep memory consumption of structs (heap buffers in `Vec`, `HashMap`, `Arc`) for byte-budgeted LRU cache eviction in consensus stores.

#### [`utils/src/triggers.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/triggers.rs)
- **Structs**:
  - `SingleTrigger`: Async one-shot event trigger backed by `tokio::sync::watch` or `tokio::sync::Notify`.
  - `DuplexTrigger`: Bidirectional channel allowing a requester to signal a worker and await its acknowledgment.

#### [`utils/src/networking.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/networking.rs)
- **Functions**:
  - `is_routable(addr: &IpAddr) -> bool`: Filters out private RFC1918 (`10.0.0.0/8`, `192.168.0.0/16`), loopback (`127.0.0.1`), link-local, and broadcast IPs.
  - `canonical_socket_addr(addr: SocketAddr) -> SocketAddr`.

#### [`utils/src/sync/rwlock.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/sync/rwlock.rs) & [`semaphore.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/utils/src/sync/semaphore.rs)
- Debug wrappers around `parking_lot::RwLock` that log lock acquisition times and warn if locks are held for $> 200\text{ ms}$ (detecting pipeline bottlenecks).

---

### 3.3 `metrics/core` Crate
Located at `metrics/core/`. Crate name: `jio-metrics-core`.

#### [`metrics/core/src/data.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/metrics/core/src/data.rs)
- **Structs**:
  - `NodeMetricsSnapshot`:
    - `bps: f64`: Blocks Per Second over last 10s.
    - `tps: f64`: Transactions Per Second over last 10s.
    - `blue_score: u64`: Virtual selected parent blue score.
    - `daa_score: u64`: Virtual block DAA score.
    - `mempool_size: usize`: Number of pending transactions.
    - `resident_set_size_bytes: u64`: Physical memory consumed.
    - `peer_count: usize`: Active connected peers.

---

## 4. Layer 1: Cryptography

### 4.1 `crypto/hashes` Crate
Located at `crypto/hashes/`. Crate name: `jio-hashes`.

#### [`crypto/hashes/src/hash.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/hashes/src/lib.rs)
- **Type**: `Hash([u8; 32])`
- **Methods**:
  - `Hash::from_bytes([u8; 32])`, `Hash::as_bytes(&self) -> &[u8; 32]`.
  - Implements `Display` (lower-hex), `FromStr` (hex parsing), `Serialize`, `Deserialize`, `BorshSerialize`, `BorshDeserialize`, `Ord`, `Hash`.

#### [`crypto/hashes/src/hashers.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/hashes/src/hashers.rs)
- **Logic**: Domain-separated BLAKE3 hashing.
- **Rule**: Every distinct consensus data structure uses a unique domain separation tag so that hashes of headers, transactions, and blocks can never collide even if their byte serialization formats match.
- **Hashers**:
  - `BlockHashHasher`: Domain `"BlockHash"`
  - `HeaderHashHasher`: Domain `"HeaderHash"`
  - `TxHashHasher`: Domain `"TransactionHash"`
  - `TxIdHasher`: Domain `"TransactionID"`
  - `BlockLevelHashHasher`: Domain `"BlockLevelHash"`
  - `MerkleHashHasher`: Domain `"MerkleTree"`
- **Implementation**:
  ```rust
  pub struct BlockHashHasher(blake3::Hasher);
  impl BlockHashHasher {
      pub fn new() -> Self {
          Self(blake3::Hasher::new_keyed(&blake3::derive_key("BlockHash", &[])))
      }
      pub fn update(&mut self, data: &[u8]) { self.0.update(data); }
      pub fn finalize(self) -> Hash { Hash(*self.0.finalize().as_bytes()) }
  }
  ```

#### [`crypto/hashes/src/pow_hashers.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/hashes/src/pow_hashers.rs)
- **Logic**: Proof-of-Work Heavy-Hash computation:
  1. Hash header pre-pow data using `cSHAKE256` with domain `"ProofOfWorkHash"`.
  2. Compute matrix multiplication of a $64 \times 64$ matrix with the hash state vector (in `consensus/pow`).
  3. Finalize with a second cSHAKE pass to yield the final PoW hash.

---

### 4.2 `crypto/addresses` Crate
Located at `crypto/addresses/`. Crate name: `jio-addresses`.

#### [`crypto/addresses/src/lib.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/addresses/src/lib.rs)
- **Enums**:
  - `Prefix`:
    - `Jio` -> `"jio"` (Mainnet)
    - `JioTest` -> `"jiotest"` (Testnet)
    - `JioSim` -> `"jiosim"` (Simnet)
    - `JioDev` -> `"jiodev"` (Devnet)
  - `Version`:
    - `PubKey = 0x00`: 32-byte Schnorr public key output.
    - `PubKeyECDSA = 0x01`: 33-byte compressed Secp256k1 public key.
    - `ScriptHash = 0x08`: 32-byte pay-to-script-hash (P2SH).
- **Struct**:
  - `Address { pub prefix: Prefix, pub version: Version, pub payload: SmallVec<[u8; 32]> }`
  - Implements `Display` (`jio:qq...`), `FromStr`, `Serialize`, `Deserialize`.

#### [`crypto/addresses/src/bech32.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/addresses/src/bech32.rs)
- **Logic**: Custom Bech32 format with BCH error-correcting code:
  - Base32 conversion (8-bit bytes $\leftrightarrow$ 5-bit words).
  - Prefix expanded into high/low bits for the checksum calculation.
  - Generates 40-bit or 48-bit BCH checksum preventing character transpositions and substitutions.
- **Error Handling**: `AddressError::InvalidPrefix`, `AddressError::InvalidChecksum`, `AddressError::InvalidPayloadLength`.

---

### 4.3 `crypto/merkle` Crate
Located at `crypto/merkle/`. Crate name: `jio-merkle`.

#### [`crypto/merkle/src/lib.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/merkle/src/lib.rs)
- **Function**: `calc_merkle_root(leaves: &[Hash]) -> Hash`
- **Logic**:
  - If `leaves.is_empty()`, returns `Hash::from_bytes([0u8; 32])`.
  - If `leaves.len() == 1`, returns `leaves[0]`.
  - While `layer.len() > 1`:
    - Pairs elements `(layer[2*i], layer[2*i + 1])`.
    - If odd number of elements, the last element is paired with itself (Bitcoin/Kaspa standard).
    - Parent hash = `MerkleHashHasher::hash(left || right)`.
  - Returns final root hash.

---

### 4.4 `crypto/muhash` Crate
Located at `crypto/muhash/`. Crate name: `jio-muhash`.

#### [`crypto/muhash/src/u3072.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/muhash/src/u3072.rs)
- **Type**: `U3072([u64; 48])`: 3072-bit unsigned integer representing elements in the multiplicative group $\mathbb{Z}/p\mathbb{Z}$, where $p = 2^{3072} - 1103717$.
- **Operations**: Modular multiplication, modular inversion (using Extended Euclidean Algorithm or Fermat's Little Theorem).

#### [`crypto/muhash/src/lib.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/muhash/src/lib.rs)
- **Type**: `MuHash(U3072)`
- **Logic**: Additive Multi-Set Hash used for incremental UTXO set commitments:
  - `MuHash::new()` initializes group identity (1).
  - `add_element(&mut self, utxo_entry_bytes: &[u8])`: hashes entry to a group element in $\mathbb{Z}/p\mathbb{Z}$ and multiplies into accumulator: $\text{state} \leftarrow \text{state} \times H(\text{entry}) \pmod p$.
  - `remove_element(&mut self, utxo_entry_bytes: &[u8])`: multiplies by the modular inverse: $\text{state} \leftarrow \text{state} \times H(\text{entry})^{-1} \pmod p$.
  - `finalize(&self) -> Hash`: BLAKE3 hash of the 3072-bit accumulator, stored in `header.utxo_commitment`.
- **Property**: Commutative ($A \text{ then } B == B \text{ then } A$), allowing UTXOs added in any order to result in the exact same commitment hash.

---

### 4.5 `crypto/txscript` Crate
Located at `crypto/txscript/`. Crate name: `jio-txscript`.

#### [`crypto/txscript/src/opcodes/macros.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/txscript/src/opcodes/macros.rs)
- **Constants**:
  - `OP_CHECKSIG = 0xac`, `OP_CHECKSIGVERIFY = 0xad`, `OP_CHECKMULTISIG = 0xae`.
  - `OP_EQUAL = 0x87`, `OP_EQUALVERIFY = 0x88`, `OP_RETURN = 0x6a`.
  - `OP_DATA_1 .. OP_DATA_75`, `OP_PUSHDATA1`, `OP_PUSHDATA2`, `OP_PUSHDATA4`.

#### [`crypto/txscript/src/script_class.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/txscript/src/script_class.rs)
- **Enum**: `ScriptClass { PubKey, PubKeyECDSA, ScriptHash, NonStandard }`
- **Pattern Matchers**:
  - `PubKey`: `[OP_DATA_32, <32-byte Schnorr pubkey>, OP_CHECKSIG]`
  - `PubKeyECDSA`: `[OP_DATA_33, <33-byte ECDSA pubkey>, OP_CHECKSIG]`
  - `ScriptHash`: `[OP_HASH256, OP_DATA_32, <32-byte script hash>, OP_EQUAL]`

#### [`crypto/txscript/src/script_builder.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/txscript/src/script_builder.rs)
- Fluent builder:
  ```rust
  ScriptBuilder::new()
      .add_data(&pubkey)
      .add_op(OP_CHECKSIG)
      .drain()
  ```

#### [`crypto/txscript/src/data_stack.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/crypto/txscript/src/data_stack.rs)
- Byte array stack machine with maximum execution depth limit (default 2048 elements) and max script element size (520 bytes) to prevent memory exhaustion attacks.

---

## 5. Layer 2: Application Framework & Canonical Consensus Domain

### 5.1 `core` Crate
Located at `core/`. Crate name: `jio-core`.

#### [`core/src/service.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/core/src/service.rs)
- **Trait**:
  ```rust
  #[async_trait::async_trait]
  pub trait AsyncService: Send + Sync + 'static {
      fn ident(&self) -> &'static str;
      fn start(self: Arc<Self>) -> Box<dyn Future<Output = Result<(), ServiceError>> + Send>;
      fn stop(&self) -> Result<(), ServiceError>;
  }
  ```

#### [`core/src/core.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/core/src/core.rs)
- **Struct**: `Core`
  - `services: Mutex<Vec<Arc<dyn AsyncService>>>`
  - `shutdown_token: CancellationToken`
- **Methods**:
  - `bind(&self, service: Arc<dyn AsyncService>)`: registers a service.
  - `start(&self)`: iterates over registered services in registration order and calls `start()`.
  - `stop(&self)`: triggers `shutdown_token` and calls `stop()` in reverse order.
  - `join(&self)`: awaits `shutdown_token.cancelled()`.

#### [`core/src/signals.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/core/src/signals.rs)
- Listens for `tokio::signal::ctrl_c()` and Unix `SIGTERM`. Upon receipt, logs `"Shutdown signal received"` and triggers the root cancellation token.

#### [`core/src/log/logger.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/core/src/log/logger.rs) & [`appender.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/core/src/log/appender.rs)
- Configures structured logging via `env_logger` / `tracing` with daily rotating log files under `~/.jiopad/<network>/logs/jiopad.log`.

---

### 5.2 `consensus/core` Crate
Located at `consensus/core/`. Crate name: `jio-consensus-core`.

#### [`consensus/core/src/header.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/header.rs)
- **Struct**:
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
  pub struct Header {
      pub version: u16,
      pub parents_by_level: Vec<Vec<Hash>>,
      pub hash_merkle_root: Hash,
      pub accepted_id_merkle_root: Hash,
      pub utxo_commitment: Hash,
      pub timestamp: u64,
      pub bits: u32,
      pub nonce: u64,
      pub daa_score: u64,
      pub blue_work: U256,
      pub blue_score: u64,
      pub pruning_point: Hash,
  }
  ```
- **Methods**:
  - `hash(&self) -> Hash`: Computes header hash via `HeaderHashHasher`.
  - `direct_parents(&self) -> &[Hash]`: Returns `parents_by_level[0]`.

#### [`consensus/core/src/tx.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/tx.rs)
- **Structs**:
  - `TransactionOutpoint { pub transaction_id: Hash, pub index: u32 }`
  - `UtxoEntry { pub amount: u64, pub script_public_key: ScriptPublicKey, pub block_daa_score: u64, pub is_coinbase: bool }`
  - `TxInput { pub previous_outpoint: TransactionOutpoint, pub signature_script: Vec<u8>, pub sequence: u64, pub sig_op_count: u8 }`
  - `TxOutput { pub value: u64, pub script_public_key: ScriptPublicKey }`
  - `Transaction { pub version: u16, pub inputs: Vec<TxInput>, pub outputs: Vec<TxOutput>, pub lock_time: u64, pub subnetwork_id: SubnetworkId, pub gas: u64, pub payload: Vec<u8>, pub mass: u64 }`
- **Methods**:
  - `id(&self) -> Hash`: Computes transaction ID via `TxIdHasher` (excluding signatures).
  - `hash(&self) -> Hash`: Computes full transaction hash via `TxHashHasher` (including signatures).

#### [`consensus/core/src/block.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/block.rs)
- **Struct**:
  ```rust
  pub struct Block {
      pub header: Header,
      pub transactions: Vec<Transaction>,
  }
  pub struct BlockTemplate {
      pub block: Block,
      pub selected_parent_timestamp: u64,
      pub selected_parent_daa_score: u64,
      pub is_synced: bool,
  }
  ```

#### [`consensus/core/src/utxo/utxo_diff.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/utxo/utxo_diff.rs)
- **Struct**:
  ```rust
  pub struct UtxoDiff {
      pub to_add: UtxoCollection,
      pub to_remove: UtxoCollection,
  }
  ```
- **Logic**:
  - `with_diff(&self, other: &UtxoDiff) -> Result<UtxoDiff, UtxoDiffError>`: Combines two diffs sequentially.
  - Rules: An outpoint added in `self` and removed in `other` is erased from both; an outpoint removed in `self` cannot be removed in `other`.

#### [`consensus/core/src/config/params.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/config/params.rs)
- **Struct**:
  ```rust
  pub struct Params {
      pub dns_seeders: &'static [&'static str],
      pub net: NetworkType,
      pub name: &'static str,
      pub genesis: GenesisBlock,
      pub ghostdag_k: u64,                      // e.g. 18 for 1 BPS, 32-64 for 10 BPS
      pub target_time_per_block: u64,           // milliseconds (e.g. 1000ms for 1 BPS, 100ms for 10 BPS)
      pub max_block_mass: u64,                  // e.g. 500_000
      pub max_block_parents: u8,                // e.g. 10
      pub difficulty_window_size: usize,        // e.g. 2641 blocks
      pub coinbase_maturity: u64,               // e.g. 100 DAA score
      pub pruning_depth: u64,                   // e.g. 185_798 blocks
      pub finality_depth: u64,                  // e.g. 86_400 blocks
  }
  ```

#### [`consensus/core/src/config/genesis.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/config/genesis.rs)
- Hardcoded genesis header, hash, timestamp, bits, and empty transaction set for each network type (`Mainnet`, `Testnet`, `Simnet`, `Devnet`).

#### [`consensus/core/src/api/mod.rs`](file:///c:/Users/LAKSHITA/Desktop/rusty-jio/consensus/core/src/api/mod.rs)
- **Trait**: **`ConsensusApi`**
  ```rust
  pub trait ConsensusApi: Send + Sync {
      fn build_block_template(&self, miner_data: MinerData, tx_selector: Box<dyn TxSelector>) -> Result<BlockTemplate, RuleError>;
      fn validate_and_insert_block(&self, block: Block) -> BlockValidationFuture;
      fn get_block(&self, hash: Hash) -> Result<Block, BlockError>;
      fn get_header(&self, hash: Hash) -> Result<Arc<Header>, BlockError>;
      fn get_virtual_daa_score(&self) -> u64;
      fn get_virtual_bits(&self) -> u32;
      fn get_tips(&self) -> Vec<Hash>;
      fn get_virtual_utxo_view(&self) -> Box<dyn UtxoView>;
  }
  ```

---

## 6. Phase 1 Implementation Order & Execution Plan

To execute Phase 1 without blocker bottlenecks, implement the files in this strict sequential order:

```
Step 1.1: Root manifests (Cargo.toml, .rustfmt.toml, clippy.toml, check scripts)
   │
Step 1.2: math crate (math/src/uint.rs -> int.rs -> wasm.rs -> lib.rs)
   │
Step 1.3: utils crate (utils/src/hex.rs -> triggers.rs -> mem_size.rs -> networking.rs)
   │
Step 1.4: metrics/core (metrics/core/src/data.rs -> lib.rs)
   │
Step 1.5: crypto/hashes (crypto/hashes/src/hash.rs -> hashers.rs -> pow_hashers.rs)
   │
Step 1.6: crypto/addresses (crypto/addresses/src/bech32.rs -> lib.rs)
   │
Step 1.7: crypto/merkle & crypto/muhash (u3072.rs -> lib.rs)
   │
Step 1.8: crypto/txscript (opcodes -> script_class.rs -> script_builder.rs -> data_stack.rs)
   │
Step 1.9: core crate (service.rs -> core.rs -> signals.rs -> task/runtime.rs -> log/logger.rs)
   │
Step 1.10: consensus/core (header.rs -> tx.rs -> block.rs -> utxo_diff.rs -> params.rs -> api/mod.rs)
```

---

## 7. Phase 1 Definition of Done

Before declaring Phase 1 complete and moving to Phase 2 (Storage & Consensus Pipeline), the following gates must pass:

1. **Compilation**: `cargo build --workspace` compiles cleanly with zero warnings (`-D warnings`).
2. **Formatting & Lints**:
   - `cargo fmt --all -- --check` passes without diffs.
   - `cargo clippy --workspace --all-targets -- -D warnings` passes without warnings.
3. **Unit Tests**:
   - `math::uint`: 100% test coverage for arithmetic overflow, compact target encoding, and division.
   - `crypto::hashes`: Verification vectors matching BLAKE3 reference values.
   - `crypto::addresses`: Encode/decode test vectors for `jio:`, `jiotest:` with BCH checksum validation.
   - `crypto::muhash`: Identity tests verifying commutativity: $H(A) \times H(B) == H(B) \times H(A)$.
   - `consensus/core::header`: Serialization round-trip tests (Borsh and Serde JSON).
4. **Lifecycle Test**:
   - A standalone binary test spins up `core::Core`, binds an example `AsyncService`, verifies clean startup, triggers `ctrl_c`, and verifies graceful shutdown with exit code 0.
