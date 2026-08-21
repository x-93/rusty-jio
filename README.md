# Rusty-Jio

[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build & Test](https://img.shields.io/badge/tests-passing-brightgreen.svg)](https://github.com/x-93/rusty-jio)

**Rusty-Jio** is a high-throughput, low-latency **BlockDAG consensus node** and distributed network implementation written in Rust. Designed for sub-second block times and high parallel transaction capacity, Rusty-Jio leverages GHOSTDAG consensus principles, BLAKE3 cryptographic domain separation, and zero-allocation multi-limb big-integer arithmetic.

---

## 🌟 Key Architecture & Design Principles

```
┌────────────────────────────────────────────────────────┐
│  Layer 8: Full-Node Daemon (jiod), CLI, RPC (wRPC/gRPC)│
├────────────────────────────────────────────────────────┤
│  Layer 7: P2P Networking, Protocol Flows & Sync Engine │
├────────────────────────────────────────────────────────┤
│  Layer 6: BlockDAG Consensus (GHOSTDAG, Reachability)  │
├────────────────────────────────────────────────────────┤
│  Layer 5: State & Indexing (UTXO Set, Virtual State)   │
├────────────────────────────────────────────────────────┤
│  Layer 4: Storage Engine (Database, RocksDB backend)   │
├────────────────────────────────────────────────────────┤
│  Layer 3: Cryptography & TxScript Engine               │
├────────────────────────────────────────────────────────┤
│  Layer 2: Core Domain Types (Block, Header, Tx)        │
├────────────────────────────────────────────────────────┤
│  Layer 1: Foundations (Math, Hashes, Utils)            │
└────────────────────────────────────────────────────────┘
```

* **Pure BLAKE3 Domain Separation**: Eliminates cross-protocol type-confusion attacks via zero-overhead `new_derive_key` domain separation.
* **Zero-Allocation Fixed-Width Math**: High-performance multi-limb [`Uint128`, `Uint192`, `Uint256`] arithmetic optimized for difficulty adjustment algorithms (DAA) and Proof-of-Work target evaluations.
* **Modular Workspace**: Cleanly isolated crates enabling independent reuse across node daemons, lightweight SPV clients, mining software, and WebAssembly (WASM) browser wallets.

---

## 📦 Implemented Crates

### 1. `jio-math` (`math/`)
* **Multi-Precision Arithmetic Engine**: Fast `[u64; N]` multi-limb big integer generation via `construct_uint!(Name, Limbs)`.
* **Types**: [`Uint128`](math/src/uint.rs), [`Uint192`](math/src/uint.rs), and [`Uint256`](math/src/uint.rs).
* **Operations**: Multi-limb addition, subtraction, multiplication with carry/borrow propagation (`carrying_add`, `borrowing_sub`, `carrying_mul`), bit shifts across limb boundaries, and float (`f64`) conversions for dynamic difficulty adjustment.
* **WASM / JS Interop**: Direct conversions to/from JavaScript `BigInt` and hex parsing.

### 2. `jio-hashes` (`crypto/hashes/`)
* **Domain-Separated BLAKE3 Engine**: Native initialization vectors generated from cryptographically isolated context strings:
  * `TransactionHash` (`"TransactionHash"`)
  * `TransactionID` (`"TransactionID"`) — Malleability-proof transaction identifier
  * `TransactionSigningHash` (`"TransactionSigningHash"`) — Sighash calculations for signature verification
  * `BlockHash` (`"BlockHash"`)
  * `ProofOfWorkHash` (`"ProofOfWorkHash"`)
  * `MerkleBranchHash` (`"MerkleBranchHash"`)
  * `MuHashElementHash` / `MuHashFinalizeHash`
  * `PersonalMessageSigningHash` (`"PersonalMessageSigningHash"`)
* **`Hash` Type (`[u8; 32]`)**:
  * Optimized 64-bit word iterator (`to_le_u64`, `from_le_u64`, `iter_le_u64`) for fast SipHash operations in hash tables.
  * Native serialization support for `Borsh`, `Serde`, `Hex`, and `WASM`.

### 3. `jio-utils` (`utils/`)
* **Utilities & Memory Management**:
  * Fast SIMD hex formatting via `faster-hex` (`ToHex`, `FromHex` traits).
  * In-memory object size estimation via `MemSizeEstimator`.
  * Macro helpers for fixed-length byte reference serialization.

---

## 📁 Repository Structure

```
rusty-jio/
├── math/              # Big integer arithmetic & precision math (jio-math)
├── crypto/
│   ├── hashes/        # BLAKE3 domain-separated hashing (jio-hashes)
│   ├── addresses/     # Bech32 address encoding & prefixes
│   ├── merkle/        # Transaction & block ID Merkle trees
│   ├── muhash/        # Rolling UTXO set multi-set hashes
│   └── txscript/      # Script execution & opcode engine
├── core/              # Core domain structures (Blocks, Headers, Transactions)
├── consensus/         # GHOSTDAG consensus, DAG reachability & PoW validation
├── database/          # Persistent database traits and RocksDB backend
├── indexes/           # UTXO & transaction indexers
├── protocol/          # P2P wire protocol and networking flows
├── rpc/               # gRPC and WebSocket RPC (wRPC) interfaces
├── daemon/            # Full-node runner executable (jiod)
├── wallet/            # Key generation, BIP32 derivation, and transaction building
├── utils/             # Shared system utilities & traits (jio-utils)
└── wasm/              # WebAssembly SDK for browser & Node.js
```

---

## 🚀 Getting Started

### Prerequisites
* **Rust**: Ensure you have the latest stable Rust toolchain installed (edition 2021, MSRV 1.79.0+).
  ```bash
  rustup update stable
  ```

### Building the Workspace
To build all workspace packages in release mode:
```bash
cargo build --workspace --release
```

### Running Tests
Execute the full unit and integration test suites:
```bash
cargo test --workspace
```

---

## 🛡️ Cryptographic Standards

| Component | Standard / Algorithm |
| :--- | :--- |
| **Cryptographic Hashing** | BLAKE3 (Domain Separated via `derive_key`) |
| **Proof-of-Work Evaluation** | $Hash_{256} \le Target_{256}$ |
| **Serialization** | Borsh (Binary) & Serde (JSON / Hex) |
| **Hex Encoding** | SIMD AVX2 / NEON accelerated `faster-hex` |

---

## 📄 License

Licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.
