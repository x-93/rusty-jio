# rusty-jio

A high-performance modular blockchain / cryptocurrency implementation in Rust.

## Phase 1 Workspace Modules

- **`core`**: Core runtime orchestration, service lifecycle management, signals, panic hooks, tasks, and structured logging.
- **`utils`**: High-performance utilities including hex codecs, custom `serde_bytes` serializers, channel synchronization, and triggers.
- **`math`**: Big-number arithmetic (`Uint128`, `Uint192`, `Uint256`), safe arithmetic helpers, and WASM bindings.
- **`crypto/hashes`**: Standardized 32-byte `Hash` type, `Hasher` trait, Blake2b, SHA256, Hash256, Ripemd160, and PoW hashers.
- **`crypto/txscript`**: Script opcodes, fluent `ScriptBuilder`, script classes (P2PK, P2PKH, P2SH, Multisig), evaluation data stack, and script error types.
- **`crypto/addresses`**: Bech32/Bech32m address encoding and decoding with support for network prefixes (`jio`, `jiotest`, `jiodev`, `jiosim`).

## Building & Testing

```bash
cargo check --workspace
cargo test --workspace
```
