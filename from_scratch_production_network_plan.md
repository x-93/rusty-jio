# Master Engineering Blueprint: Building a BlockDAG Network From Scratch

> **Goal:** Build an asynchronous, production-grade BlockDAG full-node network from zero lines of code to Mainnet release.  
> **Target Architecture:** Pipelined BlockDAG engine (GHOSTDAG consensus, RocksDB storage, multiplexed P2P flows, dual gRPC/wRPC gateways).  
> **Guiding Principle:** **Strict Inside-Out Construction** — A system you cannot simulate in memory will never survive the open internet. Never write networking code before your consensus state machine is mathematically proven.

---

## The Evolutionary Roadmap Overview

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ PHASE 0: Workspace Foundation & Toolchain (Empty Repo -> Cargo Workspace)                             │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 1: Domain Kernel & Cryptography (Hashes, Keys, Serializers, Transactions, Scripts)              │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 2: Storage Engine & State Persistence (RocksDB Column Families, LRU Caches, Batch Writers)      │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 3: BlockDAG Algorithms & Reachability (Interval Trees, GHOSTDAG Blue-Set Election, DAA)         │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 4: Pipelined Consensus Validation & Mempool (Headers -> Deps -> Body -> Virtual -> Pruning)     │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 5: In-Process Virtual-Time Simulation Framework (Synthetic 100-Node Stress Tests, Reorgs)       │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 6: P2P Networking & Multiplexed Flows (Protobuf Wire, Handshake, Block/Tx Relay, IBD Sync)       │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 7: Ingress RPC & Notification Gateways (gRPC Port 16110, WebSocket wRPC Borsh/JSON, Pub/Sub)    │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 8: Client Ecosystem (BIP32/39 HD Wallet, CLI, WebAssembly SDK)                                  │
└───────────────────────────────────┬────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────────────────────────────────────┐
│ PHASE 9: Production Hardening, Testnets & Genesis Ceremony (Devnet -> Public Testnet -> Mainnet)       │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Workspace Foundation & Architecture Setup

### 1. Objective
Establish a clean, multi-crate Rust workspace with strict unidirectional dependency boundaries.

### 2. Workspace Layout
Create root `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = [
    "core",
    "crypto/hashes",
    "crypto/addresses",
    "crypto/txscript",
    "consensus/core",
    "consensus/pow",
    "consensus",
    "database",
    "indexes",
    "mining",
    "protocol/p2p",
    "protocol/flows",
    "rpc/core",
    "rpc/grpc",
    "rpc/wrpc",
    "daemon",
    "simulation",
    "wallet/core",
    "wallet/cli",
    "wasm",
]
```

### 3. Verification Gate
* `cargo check --workspace` compiles clean with zero warnings.
* Linter rules configured in `.clippy.toml` and `.rustfmt.toml`.

---

## Phase 1: Domain Kernel & Cryptography

### 1. Objective
Build pure, deterministic data primitives and cryptographic verification logic. No I/O, no async runtimes, no database.

### 2. Crates & Logic
* **`crypto/hashes`**:
  - Blake2b 256-bit hash wrapper (`Hash([u8; 32])`).
  - Merkle Tree computation for transaction trees and accepted IDs.
  - Multi-set hash (`MuHash`) for fast, incremental UTXO set commitments.
* **`crypto/addresses`**:
  - Schnorr (BIP-340) and ECDSA public key representations on curve `secp256k1`.
  - Bech32 address encoding/decoding with custom network prefixes (`mycoin:q...`).
* **`crypto/txscript`**:
  - Forth-like stack execution engine (Opcodes: `OP_CHECKSIG`, `OP_EQUALVERIFY`, `OP_RETURN`).
  - Standard script templates: Pay-to-PubKey (P2PK), Pay-to-Script-Hash (P2SH).
* **`consensus/core`**:
  - `Transaction`, `TxInput`, `TxOutput`, `Outpoint`.
  - `BlockHeader`: Parents vector (`Vec<Hash>`), timestamp, bits (difficulty), nonce, DAA score, blue score, blue work.
  - `Block`: Header + Transaction vector.
  - Canonical binary serialization using `borsh` or `bincode`.

### 3. Verification Gate
* Unit tests verify:
  1. Transaction serialization is byte-for-byte deterministic.
  2. Schnorr signature creation and verification succeeds.
  3. Script engine executes valid scripts and rejects invalid/tampered scripts.

---

## Phase 2: Storage Layer & State Representation

### 1. Objective
Create an embedded, atomic persistence engine with LRU memory caching over RocksDB.

### 2. Crates & Logic
* **`database`**:
  - Embedded RocksDB initialization with configured Column Families:
    - `CF_HEADERS`: Stores block headers by block hash.
    - `CF_BODIES`: Stores full transactions by block hash.
    - `CF_DAG`: Stores parent/child links and blue work metrics.
    - `CF_UTXO`: Stores unspent transaction outputs (`Outpoint -> UtxoEntry`).
    - `CF_META`: Network state, pruning point hash, tips.
  - **`BatchDbWriter`**: Atomic batch commits (`rocksdb::WriteBatch`) ensuring all changes for a block are written transactionally.
  - **`CachedDbAccess<K, V>`**: Generic LRU cache wrapping column family lookups to eliminate disk I/O on hot reads.

### 3. Verification Gate
* Database crash-recovery tests: Simulate process termination during block writes; ensure on restart the database is never left in a corrupted or half-written state.

---

## Phase 3: BlockDAG Algorithms & Reachability Engine

### 1. Objective
Implement the mathematical core of the BlockDAG: GHOSTDAG ordering, $O(1)$ reachability, and difficulty adjustment.

### 2. Crates & Logic
* **`consensus/reachability`**:
  - **Interval-Tree Reachability**:
    Assigns an integer interval $[start, end]$ to each block on the selected parent tree.
    Block $A$ is an ancestor of $B$ $\iff$ $B.interval \subseteq A.interval$.
    Query time: **$O(1)$ constant time**.
* **`consensus/ghostdag`**:
  - **Selected Parent Election**: Choose the parent with maximum cumulative blue work.
  - **Blue Set Election**: For candidate blocks in parallel anticone, check if anticone size $\le k$. If yes $\rightarrow$ marked **Blue** (cooperative, rewarded); if no $\rightarrow$ marked **Red** (delayed/adversarial).
  - Calculate `blue_work` and `blue_score`.
* **`consensus/pow`**:
  - Difficulty Adjustment Algorithm (DAA): Sliding window sampling (e.g. past 2,641 blocks) targeting constant block time.
  - PoW hash verification (Blake2b matrix multiplication or target difficulty comparison).

### 3. Verification Gate
* Property-based tests (QuickCheck): Generate random DAG topologies, run GHOSTDAG, and verify:
  1. Selected parent chain is strictly monotonic.
  2. Anticone bound $k$ is never violated by the Blue set.
  3. Reachability queries match exact graph traversal results with 100% precision.

---

## Phase 4: Pipelined Consensus Validation & Mempool

### 1. Objective
Build an asynchronous, non-blocking pipeline that ingests blocks, resolves parent dependencies, validates transactions, and updates the virtual state.

### 2. Pipeline Architecture
```
Incoming Block
      │
      ▼
[Stage 1: Header Processor]  ──► Validate PoW bits, PMT timestamp ──► Persist Header
      │
      ▼
[Stage 2: Deps Manager]      ──► Check parents exist? If missing ──► Hold in staging queue
      │
      ▼
[Stage 3: Body Processor]    ──► Validate Merkle roots, Tx mass, Schnorr sigs (Rayon pool)
      │
      ▼
[Stage 4: Virtual Processor] ──► Run GHOSTDAG, apply UTXO diffs, mature coinbase rewards
      │
      ▼
[Stage 5: Pruning Processor] ──► Update pruning point, prune expired historic bodies
```

* **`mining`**:
  - **Mempool**: Stores unconfirmed transactions, ranks by fee-per-mass, rejects double-spending inputs.
  - **Block Template Builder**: Selects top transactions fitting the block mass budget and builds candidate block headers for miners.

### 3. Verification Gate
* End-to-end integration test: Feed a sequence of 5,000 blocks into the pipeline. Verify that the virtual tip correctly tracks the highest blue work tip and balances match expected coinbase rewards.

---

## Phase 5: In-Process Virtual-Time Simulation Framework

### 1. Objective
Test multi-node consensus dynamics, network splits, and reorganizations **before writing a single line of network socket code**.

### 2. Logic (Build `simulation`, like `simpa`)
* Spin up $N$ in-memory consensus instances.
* Connect instances via virtual channels with configurable delay (e.g., $100\text{ms} - 2000\text{ms}$).
* Simulate miners producing blocks at 1–10 blocks/sec.
* Introduce deliberate network partitions (split into 2 clusters, mine blocks independently, heal partition, and verify convergence).

### 3. Verification Gate
* Partition healing test: The network successfully re-converges to a single virtual state tip after a 100-block network split without panics or consensus disagreements.

---

## Phase 6: P2P Networking & Multiplexed Protocol Flows

### 1. Objective
Establish real distributed networking across the internet using multiplexed protocol flows over gRPC/Protobuf streams.

### 2. Crates & Logic
* **`protocol/p2p`**:
  - Async connection router over TCP / TLS.
  - Protobuf wire schemas (`protowire`): `BlockMessage`, `InvMessage`, `TxMessage`, `PingMessage`, `HandshakeMessage`.
* **`protocol/flows`** (Independent tasks per flow):
  - **`HandshakeFlow`**: Verifies protocol version, network ID (`mainnet`), and genesis hash match.
  - **`BlockRelayFlow`**: Broadcasts newly mined block hashes; fetches block headers and bodies.
  - **`TxRelayFlow`**: Relays unconfirmed transactions across mempools.
  - **`IBDFlow` (Initial Block Download)**:
    - Syncs headers from Genesis/Pruning point forward.
    - Downloads UTXO commitment proof at the pruning point.
    - Syncs remaining block bodies up to the virtual tip.
* **`components/addressmanager`**:
  - Manages peer IP book, connection attempts, ban scores, and DNS seed discovery.

### 3. Verification Gate
* Multi-machine test: Run Node A on Machine 1, Node B on Machine 2. Mine blocks on Node A; confirm Node B receives, validates, and incorporates blocks within $<1\text{s}$.

---

## Phase 7: Ingress RPC Gateways & Pub/Sub Subsystem

### 1. Objective
Expose the node to external clients: mining software, wallets, block explorers, and exchanges.

### 2. Crates & Logic
* **`rpc/grpc` (Port 16110)**:
  - High-throughput gRPC service (HTTP/2) for mining software (`GetBlockTemplate`, `SubmitBlock`) and enterprise backends.
* **`rpc/wrpc` (Ports 17110/18110)**:
  - WebSocket RPC server supporting **Borsh binary serialization** (zero-copy for browser/WASM apps) and **JSON-RPC**.
* **`notify`**:
  - Publish-subscribe event bus broadcasting real-time events:
    - `BlockAddedNotification`
    - `UtxosChangedNotification` (address-specific filtering)
    - `VirtualSelectedParentChainChanged`

### 3. Verification Gate
* Use `curl` / WebSocket clients to subscribe to address notifications and submit a transaction; verify notification emits instantly on block acceptance.

---

## Phase 8: Client Ecosystem & Wallets

### 1. Objective
Provide secure user tooling for key management and transaction construction.

### 2. Crates & Logic
* **`wallet/core`**:
  - **BIP39 Mnemonic**: 12/24-word seed phrase generation.
  - **BIP32/BIP44 Key Derivation**: Path `m/44'/111111'/0'/0/i`.
  - **UTXO Tracker**: Queries node via RPC for unspent outputs.
  - **Transaction Constructor**: Coin selection algorithm, fee estimator, mass calculator, and Schnorr signature signing.
* **`wallet/cli`**: Command-line interface (`create-wallet`, `send`, `balance`, `dump-unspent`).
* **`wasm`**: Compile `wallet/core` to WebAssembly for browser wallets.

### 3. Verification Gate
* CLI Wallet creates address $A$ and $B$, funds $A$, sends $X$ coins to $B$, node accepts transaction, $B$ detects confirmed balance.

---

## Phase 9: Hardening, Testnets & Genesis Launch

### 1. Objective
Transition the codebase into a battle-tested, secure, production network.

### 2. Launch Sequence

| Stage | Action | Success Metric |
|---|---|---|
| **9.1 Graceful Shutdown** | Implement multi-stage shutdown (`Ingress` $\rightarrow$ `Network` $\rightarrow$ `Consensus` $\rightarrow$ `RocksDB Flush`) | Zero database corruption or socket leaks on `SIGINT` |
| **9.2 Fuzz Testing** | Run `cargo-fuzz` on wire deserializers and script execution engines | 0 crashes over 10M executions |
| **9.3 Private Devnet** | Run 5 nodes + 2 miners on isolated cloud servers | Sustained 1-10 BPS over 7 days |
| **9.4 Public Testnet** | Launch testnet with DNS seeders, publish testnet binaries, run public faucet | Community members join and sync without forks |
| **9.5 Genesis Ceremony** | 1. Choose immutable network launch timestamp.<br/>2. Embed Genesis coinbase message.<br/>3. Compute Genesis hash and hardcode in `params.rs`.<br/>4. Tag `v1.0.0` release. | Mainnet Block 0 mined successfully |

---

*Master Architecture Blueprint for Building a Production-Level BlockDAG Network.*
