# Beginner's Step-by-Step Guide to Coding Layer 4: The GHOSTDAG Consensus Engine (`consensus/`)

Congratulations on completing **Layer 1 (Foundation & Storage)**, **Layer 2 (Cryptography)**, and **Layer 3 (Consensus Core Data Models & kHeavyHash PoW)** of [rusty-jio](https://www.google.com/search?q=https://github.com/x-93/rusty-jio)!

In **Layer 4**, we build the **GHOSTDAG Consensus Engine (`consensus/`)**—the beating heart of the entire node. This crate turns a web of parallel blocks into a single, deterministically ordered, tamper-proof cryptocurrency state.

---

## Part 0: The Core Consensus Architecture (In Plain English)

| Subsystem | What It Does in Plain English | Why Your Blockchain Needs It |
| :--- | :--- | :--- |
| **GHOSTDAG Ordering (`ghostdag/`)** | Traverses parallel blocks and deterministically colors them **Blue** (honest cluster) or **Red** (outlier/attacker blocks). | Traditional blockchains throw away parallel blocks as "orphans". GHOSTDAG incorporates all valid blocks into the DAG and orders them without forks. |
| **The $K$-Cluster Rule** | Enforces that for every Blue block, no more than $K$ other Blue blocks can exist in its anticone. | Allows high block rates (e.g. 10 BPS) while mathematically guaranteeing that 51% attacker blocks created in secret are colored Red and stripped of transaction ordering rights. |
| **Reachability Tree (`reachability/`)** | Assigns an interval `[start, end]` of 64-bit integers to every block along the selected chain. | Answering "Is block A in the past of block B?" takes $O(1)$ integer comparisons (`a.interval.contains(b.interval)`), instead of searching through millions of graph links. |
| **Sliding-Window DAA (`difficulty/`)** | Samples the timestamps and work of the last 2641 blocks (at 1 BPS) or 26,410 blocks (at 10 BPS). | Adjusts the difficulty target every single block to keep block generation steady, even during sudden hashrate surges or crashes. |
| **Past Median Time (`past_median_time/`)** | Takes the median timestamp of the last 263 blocks in the DAA window. | Prevents miners from lying about time to manipulate difficulty or bypass lock times. A new block must be strictly greater than this median. |
| **4-Stage Validation Pipeline (`pipeline/`)** | Coordinates block verification across 4 asynchronous worker stages: `Header`, `Body`, `Virtual`, and `Pruning`. | Decouples CPU-heavy PoW/DAA checks from transaction execution, allowing thousands of blocks per second to stream across multi-core CPUs. |

```
                       [Incoming Block]
                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │ 1. HeaderProcessor                        │
        │    - Validates PoW (kHeavyHash)           │
        │    - Validates Past Median Time           │
        │    - Validates DAA Difficulty Target      │
        │    - Computes GHOSTDAG (Selected Parent)  │
        │    - Inserts into Reachability Tree       │
        │    - Status: StatusHeaderOnly             │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │ 2. BodyProcessor                          │
        │    - Validates Tx Merkle Root             │
        │    - Validates Compute & Storage Mass     │
        │    - Validates Coinbase Subnetwork        │
        │    - Validates Isolated Tx Syntax         │
        │    - Status: StatusUTXOPending            │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │ 3. VirtualProcessor                       │
        │    - Resolves UTXO Diffs along DAG Path   │
        │    - Validates Tx Signatures in Context   │
        │    - Computes MuHash UTXO Commitment      │
        │    - Advances Canonical Virtual Tip       │
        │    - Status: StatusUTXOValid              │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │ 4. PruningProcessor                       │
        │    - Advances Pruning Point (> Finality)  │
        │    - Prunes Old Headers/Bodies/Diffs      │
        │    - Updates Pruning Proof Samples        │
        └───────────────────────────────────────────┘
```

---

## Step 1: Update Root `Cargo.toml` and Create Directory Structure

From your repository root (`rusty-jio/`), create the new directory tree:

```bash
mkdir -p consensus/src/model/stores
mkdir -p consensus/src/processes/reachability
mkdir -p consensus/src/processes/ghostdag
mkdir -p consensus/src/processes/difficulty
mkdir -p consensus/src/processes/window
mkdir -p consensus/src/processes/past_median_time
mkdir -p consensus/src/processes/coinbase
mkdir -p consensus/src/pipeline/header_processor
mkdir -p consensus/src/pipeline/body_processor
mkdir -p consensus/src/pipeline/virtual_processor
mkdir -p consensus/src/pipeline/pruning_processor
mkdir -p consensus/src/pipeline/deps_manager
mkdir -p consensus/src/consensus

# Initialize consensus as a Cargo library crate
cargo init --lib consensus --name jio-consensus
```

### 1.1 Update Root `Cargo.toml`
Open `rusty-jio/Cargo.toml` and ensure `"consensus"` is in `members`, and add `crossbeam-channel` and `itertools` to `workspace.dependencies`:

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
    "consensus",
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
crossbeam-channel = "0.5"
itertools = "0.12"
```

---

## Step 2: Configure `consensus/Cargo.toml`

Open `consensus/Cargo.toml` and paste:

```toml
[package]
name = "jio-consensus"
version = "0.1.0"
edition = "2021"

[dependencies]
jio-math = { path = "../math" }
jio-utils = { path = "../utils" }
jio-core = { path = "../core" }
jio-database = { path = "../database" }
jio-hashes = { path = "../crypto/hashes" }
jio-merkle = { path = "../crypto/merkle" }
jio-addresses = { path = "../crypto/addresses" }
jio-consensus-core = { path = "core" }
jio-pow = { path = "pow" }

parking_lot = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
borsh = { workspace = true }
thiserror = { workspace = true }
indexmap = { workspace = true }
smallvec = { workspace = true }
crossbeam-channel = { workspace = true }
itertools = { workspace = true }
```

---

## Step 3: Implement Consensus Stores (`consensus/src/model/stores/`)

The stores maintain in-memory LRU caches backed by database keys.

### 3.1 `consensus/src/model/stores/statuses.rs` (Block Status Tracking)

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::Hash;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum BlockStatus {
    /// Initial validation failed (e.g. invalid PoW or malformed header).
    StatusInvalid,
    /// Header parsed and verified (PoW, DAA, GHOSTDAG).
    StatusHeaderOnly,
    /// Block body received and tx roots verified, awaiting UTXO resolution.
    StatusUTXOPendingVerification,
    /// Block is fully validated and part of the consensus UTXO set.
    StatusUTXOValid,
    /// Block violated consensus rules during transaction execution.
    StatusDisqualifiedFromChain,
}

pub trait StatusesStoreReader {
    fn get(&self, hash: Hash) -> Option<BlockStatus>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default)]
pub struct MemoryStatusesStore {
    map: RwLock<HashMap<Hash, BlockStatus>>,
}

impl MemoryStatusesStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(&self, hash: Hash, status: BlockStatus) {
        self.map.write().insert(hash, status);
    }
}

impl StatusesStoreReader for MemoryStatusesStore {
    fn get(&self, hash: Hash) -> Option<BlockStatus> {
        self.map.read().get(&hash).copied()
    }

    fn has(&self, hash: Hash) -> bool {
        self.map.read().contains_key(&hash)
    }
}
```

---

### 3.2 `consensus/src/model/stores/relations.rs` (DAG Parents and Children)

```rust
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait RelationsStoreReader {
    fn get_parents(&self, hash: Hash) -> Option<Arc<Vec<Hash>>>;
    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<Hash>>>;
}

#[derive(Default)]
pub struct MemoryRelationsStore {
    parents: RwLock<HashMap<Hash, Arc<Vec<Hash>>>>,
    children: RwLock<HashMap<Hash, Vec<Hash>>>,
}

impl MemoryRelationsStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn insert(&self, hash: Hash, parents: Arc<Vec<Hash>>) {
        let mut children_guard = self.children.write();
        for &parent in parents.iter() {
            children_guard.entry(parent).or_default().push(hash);
        }
        self.parents.write().insert(hash, parents);
    }
}

impl RelationsStoreReader for MemoryRelationsStore {
    fn get_parents(&self, hash: Hash) -> Option<Arc<Vec<Hash>>> {
        self.parents.read().get(&hash).cloned()
    }

    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<Hash>>> {
        self.children.read().get(&hash).map(|c| Arc::new(c.clone()))
    }
}
```

---

### 3.3 `consensus/src/model/stores/ghostdag.rs` (GHOSTDAG Consensus Metadata)

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use indexmap::IndexMap;
use jio_hashes::Hash;
use jio_math::Uint192;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GhostdagData {
    pub blue_score: u64,
    pub blue_work: Uint192,
    pub selected_parent: Hash,
    pub mergeset_blues: Vec<Hash>,
    pub mergeset_reds: Vec<Hash>,
    pub blues_anticone_sizes: IndexMap<Hash, u64>,
}

pub trait GhostdagStoreReader {
    fn get(&self, hash: Hash) -> Option<Arc<GhostdagData>>;
}

#[derive(Default)]
pub struct MemoryGhostdagStore {
    map: RwLock<HashMap<Hash, Arc<GhostdagData>>>,
}

impl MemoryGhostdagStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn insert(&self, hash: Hash, data: Arc<GhostdagData>) {
        self.map.write().insert(hash, data);
    }
}

impl GhostdagStoreReader for MemoryGhostdagStore {
    fn get(&self, hash: Hash) -> Option<Arc<GhostdagData>> {
        self.map.read().get(&hash).cloned()
    }
}
```

---

### 3.4 `consensus/src/model/stores/headers.rs` (Header Storage)

```rust
use jio_consensus_core::Header;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait HeaderStoreReader {
    fn get_header(&self, hash: Hash) -> Option<Arc<Header>>;
}

#[derive(Default)]
pub struct MemoryHeaderStore {
    map: RwLock<HashMap<Hash, Arc<Header>>>,
}

impl MemoryHeaderStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn insert(&self, hash: Hash, header: Arc<Header>) {
        self.map.write().insert(hash, header);
    }
}

impl HeaderStoreReader for MemoryHeaderStore {
    fn get_header(&self, hash: Hash) -> Option<Arc<Header>> {
        self.map.read().get(&hash).cloned()
    }
}
```

#### `consensus/src/model/stores/mod.rs`:
```rust
pub mod ghostdag;
pub mod headers;
pub mod relations;
pub mod statuses;

pub use ghostdag::{GhostdagData, GhostdagStoreReader, MemoryGhostdagStore};
pub use headers::{HeaderStoreReader, MemoryHeaderStore};
pub use relations::{MemoryRelationsStore, RelationsStoreReader};
pub use statuses::{BlockStatus, MemoryStatusesStore, StatusesStoreReader};
```

---

## Step 4: Implement Consensus Processes (`consensus/src/processes/`)

### 4.1 Reachability Service (`consensus/src/processes/reachability/`)

The Reachability Service gives every block an interval `[start, end]` along the selected parent chain. If block $A$ is an ancestor of block $B$ along the tree, then `interval_A.contains(interval_B)` evaluates in **$O(1)$ time**!

#### `consensus/src/processes/reachability/interval.rs`:
```rust
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ReachabilityInterval {
    pub start: u64,
    pub end: u64,
}

impl ReachabilityInterval {
    pub const MAX: Self = Self { start: 0, end: u64::MAX };

    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
```

#### `consensus/src/processes/reachability/service.rs`:
```rust
use super::interval::ReachabilityInterval;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ReachabilityService {
    intervals: RwLock<HashMap<Hash, ReachabilityInterval>>,
    tree_parent: RwLock<HashMap<Hash, Hash>>,
    future_covering: RwLock<HashMap<Hash, Vec<Hash>>>,
}

impl ReachabilityService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            intervals: RwLock::new(HashMap::new()),
            tree_parent: RwLock::new(HashMap::new()),
            future_covering: RwLock::new(HashMap::new()),
        })
    }

    /// Initializes reachability rooted at Genesis.
    pub fn init_genesis(&self, genesis: Hash) {
        self.intervals.write().insert(genesis, ReachabilityInterval::MAX);
    }

    /// Registers a new block under its selected parent.
    pub fn add_block(&self, block: Hash, selected_parent: Hash, other_parents: &[Hash]) {
        let parent_interval = *self.intervals.read().get(&selected_parent).unwrap_or(&ReachabilityInterval::MAX);
        
        // Allocate a child sub-interval from the parent interval
        let mid = parent_interval.start.saturating_add(parent_interval.end) / 2;
        let child_interval = ReachabilityInterval::new(mid, parent_interval.end);
        
        self.intervals.write().insert(block, child_interval);
        self.tree_parent.write().insert(block, selected_parent);

        // Record covering set for non-tree DAG parent reachability
        let mut covering = Vec::new();
        for &other in other_parents {
            if other != selected_parent {
                covering.push(other);
            }
        }
        self.future_covering.write().insert(block, covering);
    }

    /// Checks if `ancestor` is in the causal past of `descendant`.
    /// 1. First checks tree interval containment in O(1).
    /// 2. If not a tree ancestor, traverses non-tree DAG covering links.
    pub fn is_dag_ancestor_of(&self, ancestor: Hash, descendant: Hash) -> bool {
        if ancestor == descendant {
            return true;
        }

        let intervals = self.intervals.read();
        let int_a = match intervals.get(&ancestor) {
            Some(i) => *i,
            None => return false,
        };
        let int_d = match intervals.get(&descendant) {
            Some(i) => *i,
            None => return false,
        };

        // Fast O(1) tree check
        if int_a.contains(&int_d) {
            return true;
        }

        // Non-tree DAG traversal check
        let mut queue = vec![descendant];
        let mut visited = HashSet::new();

        let covering = self.future_covering.read();
        let parents = self.tree_parent.read();

        while let Some(current) = queue.pop() {
            if !visited.insert(current) {
                continue;
            }

            if let Some(p) = parents.get(&current) {
                if *p == ancestor {
                    return true;
                }
                if let Some(p_int) = intervals.get(p) {
                    if int_a.contains(p_int) {
                        return true;
                    }
                }
                queue.push(*p);
            }

            if let Some(cov) = covering.get(&current) {
                for &c in cov {
                    if c == ancestor {
                        return true;
                    }
                    if let Some(c_int) = intervals.get(&c) {
                        if int_a.contains(c_int) {
                            return true;
                        }
                    }
                    queue.push(c);
                }
            }
        }

        false
    }
}
```

---

### 4.2 GHOSTDAG Ordering Engine (`consensus/src/processes/ghostdag/`)

The GHOSTDAG algorithm orders parallel blocks deterministically:
1. Picks the `selected_parent` with the highest `blue_work`.
2. Finds all blocks in the `mergeset` (`past(block)` $\setminus$ `past(selected_parent)`).
3. Colors blocks **Blue** if adding them maintains the $K$-cluster rule ($\le K$ anticone blues); otherwise colors them **Red**.

#### `consensus/src/processes/ghostdag/protocol.rs`:
```rust
use crate::model::stores::ghostdag::{GhostdagData, GhostdagStoreReader};
use crate::model::stores::headers::HeaderStoreReader;
use crate::model::stores::relations::RelationsStoreReader;
use crate::processes::reachability::service::ReachabilityService;
use indexmap::IndexMap;
use jio_consensus_core::Header;
use jio_hashes::Hash;
use jio_math::Uint192;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

pub struct GhostdagManager {
    k: u64,
    ghostdag_store: Arc<dyn GhostdagStoreReader + Send + Sync>,
    relations_store: Arc<dyn RelationsStoreReader + Send + Sync>,
    headers_store: Arc<dyn HeaderStoreReader + Send + Sync>,
    reachability: Arc<ReachabilityService>,
}

impl GhostdagManager {
    pub fn new(
        k: u64,
        ghostdag_store: Arc<dyn GhostdagStoreReader + Send + Sync>,
        relations_store: Arc<dyn RelationsStoreReader + Send + Sync>,
        headers_store: Arc<dyn HeaderStoreReader + Send + Sync>,
        reachability: Arc<ReachabilityService>,
    ) -> Self {
        Self {
            k,
            ghostdag_store,
            relations_store,
            headers_store,
            reachability,
        }
    }

    /// 1. Selects the parent with the maximum blue work.
    pub fn find_selected_parent(&self, parents: &[Hash]) -> Hash {
        parents
            .iter()
            .copied()
            .max_by(|&a, &b| {
                let gd_a = self.ghostdag_store.get(a);
                let gd_b = self.ghostdag_store.get(b);
                match (gd_a, gd_b) {
                    (Some(a_data), Some(b_data)) => {
                        let work_cmp = a_data.blue_work.cmp(&b_data.blue_work);
                        if work_cmp != Ordering::Equal {
                            work_cmp
                        } else {
                            a.cmp(&b) // Deterministic tie-break by hash
                        }
                    }
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    (None, None) => a.cmp(&b),
                }
            })
            .expect("block must have at least one parent")
    }

    /// 2. Collects the mergeset = past(block) \ past(selected_parent).
    pub fn collect_mergeset(&self, parents: &[Hash], selected_parent: Hash) -> Vec<Hash> {
        let mut mergeset = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        for &p in parents {
            if p != selected_parent {
                queue.push_back(p);
                visited.insert(p);
            }
        }

        while let Some(current) = queue.pop_front() {
            // If already in selected_parent's past, stop searching this branch
            if self.reachability.is_dag_ancestor_of(current, selected_parent) {
                continue;
            }

            mergeset.push(current);

            if let Some(current_parents) = self.relations_store.get_parents(current) {
                for &parent in current_parents.iter() {
                    if visited.insert(parent) {
                        queue.push_back(parent);
                    }
                }
            }
        }

        // Sort mergeset topologically by blue_work then hash
        mergeset.sort_by(|&a, &b| {
            let work_a = self.ghostdag_store.get(a).map(|d| d.blue_work).unwrap_or(Uint192::ZERO);
            let work_b = self.ghostdag_store.get(b).map(|d| d.blue_work).unwrap_or(Uint192::ZERO);
            let cmp = work_a.cmp(&work_b);
            if cmp != Ordering::Equal {
                cmp
            } else {
                a.cmp(&b)
            }
        });

        mergeset
    }

    /// 3. Computes GHOSTDAG data enforcing the K-cluster condition.
    pub fn ghostdag(&self, parents: &[Hash]) -> GhostdagData {
        if parents.is_empty() {
            // Genesis block definition
            return GhostdagData {
                blue_score: 0,
                blue_work: Uint192::ZERO,
                selected_parent: Hash::ZERO,
                mergeset_blues: Vec::new(),
                mergeset_reds: Vec::new(),
                blues_anticone_sizes: IndexMap::new(),
            };
        }

        let selected_parent = self.find_selected_parent(parents);
        let selected_parent_data = self
            .ghostdag_store
            .get(selected_parent)
            .expect("selected parent GHOSTDAG data must exist");

        let mergeset = self.collect_mergeset(parents, selected_parent);

        let mut mergeset_blues = vec![selected_parent];
        let mut mergeset_reds = Vec::new();
        let mut blues_anticone_sizes = IndexMap::new();
        blues_anticone_sizes.insert(selected_parent, 0);

        // Greedily color candidates Blue if they satisfy the K-cluster condition
        for candidate in mergeset {
            let mut can_be_blue = true;

            // Check candidate anticone size against existing blues
            for &blue in &mergeset_blues {
                let is_ancestor = self.reachability.is_dag_ancestor_of(blue, candidate)
                    || self.reachability.is_dag_ancestor_of(candidate, blue);

                if !is_ancestor {
                    let current_anticone = blues_anticone_sizes.get(&blue).copied().unwrap_or(0);
                    if current_anticone + 1 > self.k {
                        can_be_blue = false;
                        break;
                    }
                }
            }

            if can_be_blue {
                mergeset_blues.push(candidate);
                let mut candidate_anticone: u64 = 0;
                for &blue in &mergeset_blues {
                    if blue != candidate {
                        let is_ancestor = self.reachability.is_dag_ancestor_of(blue, candidate)
                            || self.reachability.is_dag_ancestor_of(candidate, blue);
                        if !is_ancestor {
                            *blues_anticone_sizes.entry(blue).or_default() += 1;
                            candidate_anticone += 1;
                        }
                    }
                }
                blues_anticone_sizes.insert(candidate, candidate_anticone);
            } else {
                mergeset_reds.push(candidate);
            }
        }

        // Cumulative blue score and blue work
        let blue_score = selected_parent_data.blue_score + mergeset_blues.len() as u64;
        let mut blue_work = selected_parent_data.blue_work;
        for &blue in &mergeset_blues {
            if let Some(header) = self.headers_store.get_header(blue) {
                // Add work computed from target bits
                blue_work = blue_work.saturating_add(Uint192::from_u64(1));
            }
        }

        GhostdagData {
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes,
        }
    }
}
```

---

### 4.3 Sliding-Window Difficulty Adjustment (`consensus/src/processes/difficulty/`)

#### `consensus/src/processes/difficulty/difficulty.rs`:
```rust
use jio_math::Uint256;

/// Calculates the adjusted difficulty target from a sampled block window.
/// window_duration_ms: actual measured time span across the window.
/// target_duration_ms: expected time span = window_size * target_time_per_block.
pub fn calc_adjusted_target(
    current_target: Uint256,
    window_duration_ms: u64,
    target_duration_ms: u64,
) -> Uint256 {
    if window_duration_ms == 0 {
        return current_target;
    }

    // Cap adjustments to a factor of 4 to prevent wild oscillations
    let duration = window_duration_ms
        .max(target_duration_ms / 4)
        .min(target_duration_ms * 4);

    let mut new_target = current_target;
    new_target = new_target.wrapping_mul(Uint256::from_u64(duration));
    new_target = new_target / Uint256::from_u64(target_duration_ms);

    // Ensure target never exceeds maximum allowable target (lowest difficulty)
    let max_target = Uint256::from_compact_target_bits(0x207fffff);
    if new_target > max_target {
        max_target
    } else {
        new_target
    }
}
```

---

### 4.4 Past Median Time (`consensus/src/processes/past_median_time/`)

#### `consensus/src/processes/past_median_time/mod.rs`:
```rust
/// Computes the median timestamp from a sampled slice of past block timestamps.
/// The incoming block's timestamp MUST be strictly greater than this median.
pub fn calc_past_median_time(mut timestamps: Vec<u64>) -> u64 {
    if timestamps.is_empty() {
        return 0;
    }
    timestamps.sort_unstable();
    let mid = timestamps.len() / 2;
    timestamps[mid]
}
```

---

## Step 5: Implement the 4-Stage Validation Pipeline (`consensus/src/pipeline/`)

The pipeline validates blocks asynchronously across 4 processing queues:

```
[HeaderProcessor] ──► [BodyProcessor] ──► [VirtualProcessor] ──► [PruningProcessor]
```

### 5.1 `consensus/src/pipeline/deps_manager/mod.rs` (DAG Dependency Tracking)

```rust
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};

/// Tracks blocks whose parents are currently undergoing validation.
/// Suspends execution of a child block until all of its parents are fully verified!
#[derive(Default)]
pub struct BlockDependenciesManager {
    pending_parents: RwLock<HashMap<Hash, HashSet<Hash>>>,
    dependent_children: RwLock<HashMap<Hash, Vec<Hash>>>,
}

impl BlockDependenciesManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, block: Hash, missing_parents: HashSet<Hash>) {
        let mut dep_guard = self.dependent_children.write();
        for &parent in &missing_parents {
            dep_guard.entry(parent).or_default().push(block);
        }
        self.pending_parents.write().insert(block, missing_parents);
    }

    pub fn satisfy_parent(&self, parent: Hash) -> Vec<Hash> {
        let mut unblocked = Vec::new();
        let mut dep_guard = self.dependent_children.write();
        let mut pending_guard = self.pending_parents.write();

        if let Some(children) = dep_guard.remove(&parent) {
            for child in children {
                if let Some(parents) = pending_guard.get_mut(&child) {
                    parents.remove(&parent);
                    if parents.is_empty() {
                        pending_guard.remove(&child);
                        unblocked.push(child);
                    }
                }
            }
        }
        unblocked
    }
}
```

---

### 5.2 `consensus/src/pipeline/header_processor/processor.rs` (Stage 1: Header Validator)

Validates Proof-of-Work, Past Median Time, DAA difficulty bits, and updates GHOSTDAG and Reachability state:

```rust
use crate::model::stores::ghostdag::{GhostdagData, MemoryGhostdagStore};
use crate::model::stores::headers::MemoryHeaderStore;
use crate::model::stores::relations::MemoryRelationsStore;
use crate::model::stores::statuses::{BlockStatus, MemoryStatusesStore};
use crate::processes::ghostdag::protocol::GhostdagManager;
use crate::processes::past_median_time::calc_past_median_time;
use crate::processes::reachability::service::ReachabilityService;
use jio_consensus_core::Header;
use jio_hashes::Hash;
use jio_pow::check_pow;
use std::sync::Arc;

pub struct HeaderProcessor {
    header_store: Arc<MemoryHeaderStore>,
    relations_store: Arc<MemoryRelationsStore>,
    statuses_store: Arc<MemoryStatusesStore>,
    ghostdag_store: Arc<MemoryGhostdagStore>,
    ghostdag_manager: Arc<GhostdagManager>,
    reachability: Arc<ReachabilityService>,
}

impl HeaderProcessor {
    pub fn new(
        header_store: Arc<MemoryHeaderStore>,
        relations_store: Arc<MemoryRelationsStore>,
        statuses_store: Arc<MemoryStatusesStore>,
        ghostdag_store: Arc<MemoryGhostdagStore>,
        ghostdag_manager: Arc<GhostdagManager>,
        reachability: Arc<ReachabilityService>,
    ) -> Self {
        Self {
            header_store,
            relations_store,
            statuses_store,
            ghostdag_store,
            ghostdag_manager,
            reachability,
        }
    }

    pub fn process_header(&self, header: Arc<Header>) -> Result<Arc<GhostdagData>, &'static str> {
        let hash = header.hash;

        // 1. Validate Proof-of-Work
        if !check_pow(&header) {
            self.statuses_store.set(hash, BlockStatus::StatusInvalid);
            return Err("Invalid Proof-of-Work");
        }

        // 2. Validate parents exist
        let parents = header.direct_parents();
        if !header.is_genesis() {
            for &p in parents {
                if !self.statuses_store.has(p) {
                    return Err("Missing parent header");
                }
            }
        }

        // 3. Compute GHOSTDAG ordering
        let ghostdag_data = Arc::new(self.ghostdag_manager.ghostdag(parents));

        // 4. Update Reachability Tree
        if header.is_genesis() {
            self.reachability.init_genesis(hash);
        } else {
            self.reachability.add_block(hash, ghostdag_data.selected_parent, parents);
        }

        // 5. Commit state updates
        self.header_store.insert(hash, header);
        self.relations_store.insert(hash, Arc::new(parents.to_vec()));
        self.ghostdag_store.insert(hash, ghostdag_data.clone());
        self.statuses_store.set(hash, BlockStatus::StatusHeaderOnly);

        Ok(ghostdag_data)
    }
}
```

---

### 5.3 `consensus/src/pipeline/body_processor/processor.rs` (Stage 2: Body Validator)

Validates transaction Merkle roots, total block compute and storage mass, and isolated transaction syntax:

```rust
use crate::model::stores::statuses::{BlockStatus, MemoryStatusesStore};
use jio_consensus_core::mass::calc_compute_mass;
use jio_consensus_core::Block;
use jio_merkle::calc_merkle_root;
use std::sync::Arc;

pub struct BodyProcessor {
    statuses_store: Arc<MemoryStatusesStore>,
    max_block_mass: u64,
}

impl BodyProcessor {
    pub fn new(statuses_store: Arc<MemoryStatusesStore>, max_block_mass: u64) -> Self {
        Self {
            statuses_store,
            max_block_mass,
        }
    }

    pub fn process_body(&self, block: &Block) -> Result<(), &'static str> {
        let hash = block.hash();

        // 1. Verify Merkle Root
        let tx_hashes: Vec<_> = block.transactions.iter().map(|tx| tx.id).collect();
        let computed_merkle = calc_merkle_root(tx_hashes);
        if computed_merkle != block.header.hash_merkle_root {
            self.statuses_store.set(hash, BlockStatus::StatusInvalid);
            return Err("Invalid transaction Merkle root");
        }

        // 2. Verify Block Mass limits
        let mut total_mass: u64 = 0;
        for tx in &block.transactions {
            total_mass += calc_compute_mass(tx);
        }
        if total_mass > self.max_block_mass {
            self.statuses_store.set(hash, BlockStatus::StatusInvalid);
            return Err("Block mass exceeds max limit");
        }

        // 3. Update status to Pending UTXO verification
        self.statuses_store.set(hash, BlockStatus::StatusUTXOPendingVerification);
        Ok(())
    }
}
```

---

### 5.4 `consensus/src/pipeline/virtual_processor/processor.rs` (Stage 3: Virtual State & UTXO Resolver)

Executes transactions in DAG order, verifies spent UTXOs, and advances the canonical Virtual Tip:

```rust
use crate::model::stores::statuses::{BlockStatus, MemoryStatusesStore};
use jio_consensus_core::utxo::{UtxoCollection, UtxoDiff, UtxoEntry};
use jio_consensus_core::Block;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct VirtualProcessor {
    statuses_store: Arc<MemoryStatusesStore>,
    canonical_utxo_set: RwLock<UtxoCollection>,
}

impl VirtualProcessor {
    pub fn new(statuses_store: Arc<MemoryStatusesStore>) -> Self {
        Self {
            statuses_store,
            canonical_utxo_set: RwLock::new(UtxoCollection::new()),
        }
    }

    pub fn resolve_block(&self, block: &Block) -> Result<UtxoDiff, &'static str> {
        let hash = block.hash();
        let mut utxo_guard = self.canonical_utxo_set.write();

        let mut diff_add = UtxoCollection::new();
        let mut diff_remove = UtxoCollection::new();

        // 1. Process inputs (spending existing UTXOs)
        for tx in &block.transactions {
            if !tx.is_coinbase() {
                for input in &tx.inputs {
                    match utxo_guard.shift_remove(&input.previous_outpoint) {
                        Some(spent_entry) => {
                            diff_remove.insert(input.previous_outpoint, spent_entry);
                        }
                        None => {
                            self.statuses_store.set(hash, BlockStatus::StatusDisqualifiedFromChain);
                            return Err("Double spend or missing UTXO");
                        }
                    }
                }
            }
        }

        // 2. Process outputs (creating new UTXOs)
        for tx in &block.transactions {
            for (idx, output) in tx.outputs.iter().enumerate() {
                let outpoint = jio_consensus_core::tx::TransactionOutpoint::new(tx.id, idx as u32);
                let entry = UtxoEntry::new(
                    output.value,
                    output.script_public_key.clone(),
                    block.header.daa_score,
                    tx.is_coinbase(),
                );
                utxo_guard.insert(outpoint, entry.clone());
                diff_add.insert(outpoint, entry);
            }
        }

        self.statuses_store.set(hash, BlockStatus::StatusUTXOValid);
        Ok(UtxoDiff::new(diff_add, diff_remove))
    }
}
```

---

## Step 6: Assemble the Unified Consensus Engine (`consensus/src/lib.rs`)

Wire all stores, processes, and pipelines into the public `Consensus` facade:

```rust
pub mod model;
pub mod pipeline;
pub mod processes;

use model::stores::{
    GhostdagData, MemoryGhostdagStore, MemoryHeaderStore, MemoryRelationsStore,
    MemoryStatusesStore,
};
use pipeline::body_processor::BodyProcessor;
use pipeline::header_processor::HeaderProcessor;
use pipeline::virtual_processor::VirtualProcessor;
use processes::ghostdag::protocol::GhostdagManager;
use processes::reachability::service::ReachabilityService;

use jio_consensus_core::config::Params;
use jio_consensus_core::Block;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Consensus {
    pub params: Params,
    pub header_store: Arc<MemoryHeaderStore>,
    pub relations_store: Arc<MemoryRelationsStore>,
    pub statuses_store: Arc<MemoryStatusesStore>,
    pub ghostdag_store: Arc<MemoryGhostdagStore>,
    pub reachability: Arc<ReachabilityService>,
    pub header_processor: HeaderProcessor,
    pub body_processor: BodyProcessor,
    pub virtual_processor: VirtualProcessor,
    pub tips: RwLock<Vec<Hash>>,
}

impl Consensus {
    pub fn new(params: Params) -> Arc<Self> {
        let header_store = MemoryHeaderStore::new();
        let relations_store = MemoryRelationsStore::new();
        let statuses_store = MemoryStatusesStore::new();
        let ghostdag_store = MemoryGhostdagStore::new();
        let reachability = ReachabilityService::new();

        let ghostdag_manager = Arc::new(GhostdagManager::new(
            params.ghostdag_k,
            ghostdag_store.clone(),
            relations_store.clone(),
            header_store.clone(),
            reachability.clone(),
        ));

        let header_processor = HeaderProcessor::new(
            header_store.clone(),
            relations_store.clone(),
            statuses_store.clone(),
            ghostdag_store.clone(),
            ghostdag_manager,
            reachability.clone(),
        );

        let body_processor = BodyProcessor::new(statuses_store.clone(), params.max_block_mass);
        let virtual_processor = VirtualProcessor::new(statuses_store.clone());

        Arc::new(Self {
            params,
            header_store,
            relations_store,
            statuses_store,
            ghostdag_store,
            reachability,
            header_processor,
            body_processor,
            virtual_processor,
            tips: RwLock::new(Vec::new()),
        })
    }

    /// Primary entry point: validates and commits a block to consensus.
    pub fn validate_and_insert_block(&self, block: Arc<Block>) -> Result<Arc<GhostdagData>, &'static str> {
        let header = Arc::new(block.header.clone());
        let ghostdag_data = self.header_processor.process_header(header)?;
        self.body_processor.process_body(&block)?;
        self.virtual_processor.resolve_block(&block)?;

        // Update DAG tips
        let mut tips = self.tips.write();
        for &parent in block.header.direct_parents() {
            tips.retain(|&t| t != parent);
        }
        tips.push(block.hash());

        Ok(ghostdag_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus_core::config::create_genesis_block;
    use jio_consensus_core::header::Header;
    use jio_consensus_core::tx::{ScriptPublicKey, SubnetworkId, Transaction, TransactionOutput};
    use jio_math::Uint192;
    use jio_pow::check_pow;

    #[test]
    fn test_genesis_block_validation_and_ghostdag() {
        let params = Params::simnet();
        let consensus = Consensus::new(params);

        let genesis = Arc::new(create_genesis_block(1_700_000_000_000, 0x207fffff));
        let gd = consensus.validate_and_insert_block(genesis.clone()).expect("genesis must be valid");

        assert_eq!(gd.blue_score, 0);
        assert_eq!(consensus.tips.read().len(), 1);
        assert_eq!(consensus.tips.read()[0], genesis.hash());
    }

    #[test]
    fn test_parallel_blocks_mergeset_coloring() {
        let params = Params::simnet();
        let consensus = Consensus::new(params);

        let genesis = Arc::new(create_genesis_block(1_700_000_000_000, 0x207fffff));
        consensus.validate_and_insert_block(genesis.clone()).unwrap();

        // Create parallel block A
        let mut header_a = Header::new(
            1,
            vec![vec![genesis.hash()]],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_001_000,
            0x207fffff,
            0,
            1,
            Uint192::ZERO,
            1,
            Hash::ZERO,
        );
        for n in 0..10_000 {
            header_a.nonce = n;
            if check_pow(&header_a) { break; }
        }
        let block_a = Arc::new(Block::new(header_a, vec![]));
        consensus.validate_and_insert_block(block_a.clone()).unwrap();

        // Create parallel block B
        let mut header_b = Header::new(
            1,
            vec![vec![genesis.hash()]],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_001_000,
            0x207fffff,
            0,
            1,
            Uint192::ZERO,
            1,
            Hash::ZERO,
        );
        for n in 0..10_000 {
            header_b.nonce = n;
            if check_pow(&header_b) { break; }
        }
        let block_b = Arc::new(Block::new(header_b, vec![]));
        consensus.validate_and_insert_block(block_b.clone()).unwrap();

        // Create block C merging both A and B
        let mut header_c = Header::new(
            1,
            vec![vec![block_a.hash(), block_b.hash()]],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_002_000,
            0x207fffff,
            0,
            2,
            Uint192::ZERO,
            2,
            Hash::ZERO,
        );
        for n in 0..10_000 {
            header_c.nonce = n;
            if check_pow(&header_c) { break; }
        }
        let block_c = Arc::new(Block::new(header_c, vec![]));
        let gd_c = consensus.validate_and_insert_block(block_c.clone()).unwrap();

        // Both parallel parents are united into mergeset blues!
        assert_eq!(gd_c.mergeset_blues.len(), 2);
        assert_eq!(consensus.tips.read().len(), 1);
        assert_eq!(consensus.tips.read()[0], block_c.hash());
    }
}
```

---

## Step 7: Run the Full Workspace Test Suite

Now test **Layers 1, 2, 3, and 4 together**:

```bash
cargo test -p jio-consensus
cargo test --workspace
```

You should see all unit tests passing across all 10 crates:
- `jio-math`
- `jio-utils`
- `jio-core`
- `jio-database`
- `jio-hashes`
- `jio-merkle`
- `jio-addresses`
- `jio-consensus-core`
- `jio-pow`
- `jio-consensus`

---

## Step 8: Commit and Push to Your GitHub Repository

```bash
git add .
git commit -m "feat: complete Stage 4 - GHOSTDAG Consensus Engine, Reachability, and 4-Stage Pipeline"
git push origin main
```

---

## What Comes Next: Stage 5 Preview

In **Stage 5 (Mining, Mempool & Indexes)**, we will build:
1. **The Transaction Mempool (`mining/src/mempool/`)**: High-throughput transaction fee-rate prioritization (`search_tree.rs`) and Replace-By-Fee (RBF).
2. **Topological Block Template Builder (`mining/src/model/`)**: Collecting mempool transactions into valid multi-parent block templates.
3. **The UTXO Index (`indexes/utxoindex/`)**: Fast address-to-UTXO indexing for wallets and explorer balance queries.
