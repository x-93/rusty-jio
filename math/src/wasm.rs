//! WebAssembly bindings for `jio-math` types.
//!
//! This module exposes [`Uint256`] and [`Uint192`] to JavaScript/TypeScript
//! consumers via `wasm-bindgen`. It will be fully implemented in **Phase 8**
//! when the WASM SDK and browser wallet are built.
//!
//! For now this is a structural placeholder ensuring the feature gate
//! compiles cleanly.

// TODO(Phase 8): Expose the following to WASM:
//   - Uint256: constructor from hex string, arithmetic, comparison, formatting
//   - Uint192: constructor from hex string, arithmetic, comparison, formatting
//   - Conversion helpers between JS BigInt and Uint256/Uint192

