# Stage 1 — Foundational Math & Utilities Specification

## 1. Objective

Stage 1 builds the bedrock for the entire Rusty Jio ecosystem:
1. Setting up the root **Cargo Workspace** manifest (`Cargo.toml`).
2. Implementing the **`jio-math`** crate providing fixed-width, multi-precision unsigned integers (`Uint192` and `Uint256`).
3. Implementing the **`jio-utils`** crate providing hex encoding/decoding, asynchronous shutdown triggers, and memory tracking primitives.

---

## 2. Component Specifications

### 2.1. Root Workspace (`Cargo.toml`)
- Configured with `resolver = "2"`.
- Centralized dependency specifications (`[workspace.dependencies]`) to prevent dependency version drift across sub-crates.
- Release profile optimizations (LTO, codegen-units, panic strategies).

### 2.2. Math Crate (`math/`)
In BlockDAG consensus:
- **`Uint192` (192-bit unsigned integer)**:
  - Formed by three 64-bit limbs (`[u64; 3]`).
  - Represents accumulated blue work in GHOSTDAG.
  - Maximum value: $2^{192} - 1 \approx 6.27 \times 10^{57}$, which guarantees safety against overflow across the lifetime of the network.
- **`Uint256` (256-bit unsigned integer)**:
  - Formed by four 64-bit limbs (`[u64; 4]`).
  - Represents cryptographic hashes, PoW targets, and difficulty levels.

#### Required Capabilities:
- **Constructors & Constants**: `zero()`, `one()`, `MAX`, `from_u64()`, `from_u128()`.
- **Byte Conversions**:
  - `to_le_bytes()` and `from_le_bytes()` (little-endian byte arrays).
  - `to_be_bytes()` and `from_be_bytes()` (big-endian byte arrays).
- **Arithmetic Operations**:
  - `Add`, `Sub`, `Mul`, `Div`, `Rem` (with checked and saturating variants).
  - Bitwise operations (`BitAnd`, `BitOr`, `BitXor`, `Not`, `Shl`, `Shr`).
- **Formatting**:
  - `Display` (decimal representation).
  - `LowerHex`, `UpperHex` (hexadecimal formatting).
  - `Debug`.
- **Serialization**:
  - `serde::Serialize` and `serde::Deserialize` implementations.

### 2.3. Utilities Crate (`utils/`)
- **Hex Utilities (`utils/src/hex.rs`)**:
  - Zero-allocation or high-speed conversion between byte slices and hexadecimal strings using `faster-hex`.
  - Traits `ToHex` and `FromHex` for convenient string/byte interop.
- **Trigger Primitives (`utils/src/triggers.rs`)**:
  - Multi-consumer asynchronous shutdown coordination (`Trigger` and `Listener`).
  - Allows the full node to gracefully broadcast cancellation and shutdown signals to background actors.
- **Memory Size (`utils/src/mem_size.rs`)**:
  - `MemSize` trait to calculate heap and stack memory footprint of cached data structures.

