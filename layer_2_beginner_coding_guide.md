# Beginner's Step-by-Step Guide to Coding Layer 2: Cryptography (`crypto/`)

Congratulations on completing and committing **Stage 1 (Workspace Root, Math, Utils, Core, Database)** to [x-93/rusty-jio](https://github.com/x-93/rusty-jio)! Your repository structure is clean, idiomatic, and ready for Layer 2.

In **Layer 2**, we build the **Cryptographic Primitives** that form the security backbone of your blockchain:
1. **`crypto/hashes`**: 32-byte `Hash` type and domain-separated Blake2b-256 hashing.
2. **`crypto/merkle`**: Computing Merkle root hashes for transactions in a block.
3. **`crypto/addresses`**: Encoding public keys into user-friendly `jio:qp...` Bech32m wallet addresses.

---

## Part 0: Key Concepts in Blockchain Cryptography (In Plain English)

| Concept | What It Does | Why Your Blockchain Needs It |
| :--- | :--- | :--- |
| **`Hash([u8; 32])`** | A 32-byte fingerprint of data. | Every block, transaction, and state commitment is identified by its 32-byte hash. |
| **Domain Separation** | Salting hash functions with a unique label (e.g., `"BlockHash"`, `"TransactionID"`). | Prevents cross-type collision attacks where an attacker crafts a transaction that produces the exact same hash as a block header. |
| **Merkle Tree Root** | A single 32-byte hash summarizing hundreds of transactions. | Allows nodes to verify that a transaction is in a block without checking every other transaction. |
| **Bech32m Address** | A human-readable address format with built-in error detection (`jio:q...`). | Prevents users from accidentally sending coins to a typo address or the wrong network (e.g., testnet vs. mainnet). |

---

## Step 1: Update Root `Cargo.toml` and Create Folders

From your repository root (`rusty-jio/`), create the new crate folders:

```bash
mkdir -p crypto/hashes
mkdir -p crypto/merkle
mkdir -p crypto/addresses

# Initialize them as Cargo library crates
cargo init --lib crypto/hashes --name jio-hashes
cargo init --lib crypto/merkle --name jio-merkle
cargo init --lib crypto/addresses --name jio-addresses
```

### 1.1 Update Root `Cargo.toml`
Open `rusty-jio/Cargo.toml` and update the `members` and `workspace.dependencies`:

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
]

[workspace.dependencies]
thiserror = "1.0"
parking_lot = "0.12"
tokio = { version = "1.36", features = ["full"] }
rocksdb = "0.21"
serde = { version = "1.0", features = ["derive"] }
blake2b_simd = "1.0"
bech32 = "0.9"
```

---

## Step 2: Code Crate #1 — `crypto/hashes/`

### 2.1 Edit `crypto/hashes/Cargo.toml`
Open `crypto/hashes/Cargo.toml`:
```toml
[package]
name = "jio-hashes"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
blake2b_simd.workspace = true
```

### 2.2 Create `crypto/hashes/src/lib.rs`
Define the core 32-byte `Hash` type used across the entire workspace:

```rust
use std::fmt;

/// A 32-byte cryptographic hash identifier
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub const ZERO: Self = Self([0u8; 32]);

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
```

### 2.3 Create `crypto/hashes/src/hashers.rs`
Implement domain-separated hashers using Blake2b:

```rust
use crate::hash::Hash;
use blake2b_simd::Params;

/// A domain-separated hasher using Blake2b-256
pub struct Hasher {
    state: blake2b_simd::State,
}

impl Hasher {
    /// Create a hasher with a specific domain separation label
    pub fn new(domain_separator: &str) -> Self {
        let mut params = Params::new();
        params.hash_length(32);
        // Blake2b personal tag (up to 16 bytes)
        let personal = domain_separator.as_bytes();
        let len = personal.len().min(16);
        params.personal(&personal[..len]);

        Self {
            state: params.to_state(),
        }
    }

    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.state.update(data);
        self
    }

    pub fn finalize(self) -> Hash {
        let digest = self.state.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(digest.as_bytes());
        Hash(out)
    }
}

/// Convenience functions for common protocol hash types
pub fn hash_block(data: &[u8]) -> Hash {
    Hasher::new("BlockHash").update(data).finalize()
}

pub fn hash_transaction(data: &[u8]) -> Hash {
    Hasher::new("TransactionHash").update(data).finalize()
}

pub fn hash_transaction_id(data: &[u8]) -> Hash {
    Hasher::new("TransactionID").update(data).finalize()
}
```

### 2.4 Wire `crypto/hashes/src/lib.rs` and Add Tests
```rust
pub mod hash;
pub mod hashers;

pub use hash::Hash;
pub use hashers::{hash_block, hash_transaction, hash_transaction_id, Hasher};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation_produces_different_hashes() {
        let data = b"common payload";
        let block_h = hash_block(data);
        let tx_h = hash_transaction(data);

        // Same data, different domain tags -> hashes MUST be completely different!
        assert_ne!(block_h, tx_h);
        assert_ne!(block_h, Hash::ZERO);
    }
}
```

Test it: `cargo test -p jio-hashes`

---

## Step 3: Code Crate #2 — `crypto/merkle/`

### 3.1 Edit `crypto/merkle/Cargo.toml`
```toml
[package]
name = "jio-merkle"
version = "0.1.0"
edition = "2021"

[dependencies]
jio-hashes = { path = "../hashes" }
```

### 3.2 Create `crypto/merkle/src/lib.rs`
In a block, we need to hash all transaction IDs together into a single 32-byte `hash_merkle_root`:

```rust
use jio_hashes::{Hash, Hasher};

/// Combine two hashes using domain-separated MerkleBranchHash
pub fn hash_branch(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Hasher::new("MerkleBranchHash");
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    hasher.finalize()
}

/// Compute the Merkle root from a list of transaction hashes
pub fn calc_merkle_root(mut hashes: Vec<Hash>) -> Hash {
    if hashes.is_empty() {
        return Hash::ZERO;
    }

    while hashes.len() > 1 {
        let mut next_level = Vec::with_capacity((hashes.len() + 1) / 2);

        for chunk in hashes.chunks(2) {
            if chunk.len() == 2 {
                next_level.push(hash_branch(&chunk[0], &chunk[1]));
            } else {
                // If odd number of hashes, duplicate the last hash (Bitcoin/Kaspa standard)
                next_level.push(hash_branch(&chunk[0], &chunk[0]));
            }
        }

        hashes = next_level;
    }

    hashes[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty_and_single() {
        assert_eq!(calc_merkle_root(vec![]), Hash::ZERO);

        let h1 = Hash([1u8; 32]);
        assert_eq!(calc_merkle_root(vec![h1]), h1);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let h1 = Hash([1u8; 32]);
        let h2 = Hash([2u8; 32]);
        let h3 = Hash([3u8; 32]);

        let root = calc_merkle_root(vec![h1, h2, h3]);
        assert_ne!(root, Hash::ZERO);
        assert_ne!(root, h1);
    }
}
```

Test it: `cargo test -p jio-merkle`

---

## Step 4: Code Crate #3 — `crypto/addresses/`

### 4.1 Edit `crypto/addresses/Cargo.toml`
```toml
[package]
name = "jio-addresses"
version = "0.1.0"
edition = "2021"

[dependencies]
bech32.workspace = true
thiserror.workspace = true
```

### 4.2 Create `crypto/addresses/src/prefix.rs`
Network prefixes differentiate between Mainnet, Testnet, Simnet, and Devnet:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prefix {
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

impl Prefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            Prefix::Mainnet => "jio",
            Prefix::Testnet => "jiotest",
            Prefix::Simnet => "jiosim",
            Prefix::Devnet => "jiodev",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "jio" => Ok(Prefix::Mainnet),
            "jiotest" => Ok(Prefix::Testnet),
            "jiosim" => Ok(Prefix::Simnet),
            "jiodev" => Ok(Prefix::Devnet),
            _ => Err(format!("Unknown address prefix: {}", s)),
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

### 4.3 Create `crypto/addresses/src/address.rs`
Implement the `Address` struct with Bech32m encoding and decoding:

```rust
use crate::prefix::Prefix;
use bech32::{self, FromBase32, ToBase32, Variant};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressError {
    #[error("Bech32 encoding/decoding error: {0}")]
    Bech32(#[from] bech32::Error),
    #[error("Invalid address format")]
    InvalidFormat,
    #[error("Unknown prefix: {0}")]
    UnknownPrefix(String),
}

/// Address version indicating key type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    PubKey = 0,      // 32-byte Schnorr public key
    PubKeyECDSA = 1, // 33-byte secp256k1 public key
    ScriptHash = 8,  // P2SH multisig script
}

impl Version {
    pub fn from_u8(v: u8) -> Result<Self, AddressError> {
        match v {
            0 => Ok(Version::PubKey),
            1 => Ok(Version::PubKeyECDSA),
            8 => Ok(Version::ScriptHash),
            _ => Err(AddressError::InvalidFormat),
        }
    }
}

/// A human-readable Bech32m address (e.g., `jio:qp3d...`)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Address {
    pub prefix: Prefix,
    pub version: Version,
    pub payload: Vec<u8>,
}

impl Address {
    pub fn new(prefix: Prefix, version: Version, payload: Vec<u8>) -> Self {
        Self { prefix, version, payload }
    }

    /// Encode address into string: `<prefix>:qp...`
    pub fn encode(&self) -> Result<String, AddressError> {
        let mut data = Vec::with_capacity(1 + self.payload.len());
        data.push(self.version as u8);
        data.extend_from_slice(&self.payload);

        let encoded = bech32::encode(self.prefix.as_str(), data.to_base32(), Variant::Bech32m)?;
        // Format as prefix:payload (Jio standard)
        if let Some(pos) = encoded.find('1') {
            let (hrp, rest) = encoded.split_at(pos);
            Ok(format!("{}:{}", hrp, &rest[1..]))
        } else {
            Ok(encoded)
        }
    }

    /// Decode address string: `jio:qp...` -> Address
    pub fn decode(addr_str: &str) -> Result<Self, AddressError> {
        let parts: Vec<&str> = addr_str.split(':').collect();
        if parts.len() != 2 {
            return Err(AddressError::InvalidFormat);
        }
        let prefix = Prefix::from_str(parts[0]).map_err(AddressError::UnknownPrefix)?;

        // Reconstruct standard bech32 string with separator '1'
        let bech32_str = format!("{}1{}", parts[0], parts[1]);
        let (hrp, base32_data, variant) = bech32::decode(&bech32_str)?;

        if variant != Variant::Bech32m || hrp != prefix.as_str() {
            return Err(AddressError::InvalidFormat);
        }

        let raw_bytes: Vec<u8> = Vec::from_base32(&base32_data)?;
        if raw_bytes.is_empty() {
            return Err(AddressError::InvalidFormat);
        }

        let version = Version::from_u8(raw_bytes[0])?;
        let payload = raw_bytes[1..].to_vec();

        Ok(Self { prefix, version, payload })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.encode() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid address>"),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
```

### 4.4 Wire `crypto/addresses/src/lib.rs` and Add Tests
```rust
pub mod address;
pub mod prefix;

pub use address::{Address, AddressError, Version};
pub use prefix::Prefix;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_encode_and_decode_roundtrip() {
        let pubkey_bytes = vec![0x42; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, pubkey_bytes.clone());

        let encoded = addr.encode().expect("encoding should succeed");
        assert!(encoded.starts_with("jio:"));

        let decoded = Address::decode(&encoded).expect("decoding should succeed");
        assert_eq!(decoded.prefix, Prefix::Mainnet);
        assert_eq!(decoded.version, Version::PubKey);
        assert_eq!(decoded.payload, pubkey_bytes);
    }

    #[test]
    fn test_testnet_address_prefix() {
        let pubkey_bytes = vec![0x99; 32];
        let addr = Address::new(Prefix::Testnet, Version::PubKey, pubkey_bytes);
        let encoded = addr.encode().unwrap();
        assert!(encoded.starts_with("jiotest:"));
    }
}
```

Test it: `cargo test -p jio-addresses`

---

## Step 5: Run the Complete Workspace Test Suite

Now test **Layer 1 and Layer 2 together**:
```bash
cargo test --workspace
```

You should see all unit tests passing across all 7 crates:
- `jio-math`
- `jio-utils`
- `jio-core`
- `jio-database`
- `jio-hashes`
- `jio-merkle`
- `jio-addresses`

---

## Step 6: Commit and Push to Your GitHub Repository

Once tests pass, commit Layer 2 to your git repository:

```bash
git add .
git commit -m "feat: complete Stage 2 - Cryptographic Primitives (hashes, merkle, addresses)"
git push origin main
```
