use crate::model::stores::ghostdag::GhostdagStoreReader;
use crate::processes::reachability::ReachabilityService;
use consensus_core::{BlockHashMap, HashMapCustomHasher, KType};
use jio_hashes::Hash;
use std::collections::HashSet;

/// Finds the selected parent from a list of parent hashes according to GHOSTDAG rules:
/// 1. Maximum blue work
/// 2. Maximum blue score
/// 3. Minimum hash value (lexicographical tie-breaking)
pub fn find_selected_parent(store: &impl GhostdagStoreReader, parents: &[Hash]) -> Option<Hash> {
    if parents.is_empty() {
        return None;
    }

    let mut best_parent = parents[0];
    let (mut best_work, mut best_score) = store
        .get(best_parent)
        .map(|d| (d.blue_work, d.blue_score))
        .unwrap_or_default();

    for &parent in &parents[1..] {
        let (work, score) = store
            .get(parent)
            .map(|d| (d.blue_work, d.blue_score))
            .unwrap_or_default();

        let is_better = work > best_work
            || (work == best_work && score > best_score)
            || (work == best_work && score == best_score && parent < best_parent);

        if is_better {
            best_parent = parent;
            best_work = work;
            best_score = score;
        }
    }

    Some(best_parent)
}

/// Evaluates candidates in the topologically sorted merge set, greedily electing blocks
/// to the Blue set if adding them maintains a valid K-cluster, otherwise marking them Red.
pub fn evaluate_mergeset(
    store: &impl GhostdagStoreReader,
    reachability: &impl ReachabilityService,
    k: KType,
    selected_parent: Hash,
    sorted_mergeset: &[Hash],
) -> (Vec<Hash>, Vec<Hash>, BlockHashMap<KType>) {
    let mut mergeset_blues = Vec::new();
    let mut mergeset_reds = Vec::new();

    // Start with a clone of selected parent's anticone sizes map if available
    let mut blues_anticone_sizes: BlockHashMap<KType> =
        match store.get_blues_anticone_sizes(selected_parent) {
            Ok(sizes) => (*sizes).clone(),
            Err(_) => BlockHashMap::new(),
        };

    for &candidate in sorted_mergeset {
        // Find all blue blocks in the anticone of candidate
        let mut anticone_blues = HashSet::new();

        // 1. Check blues already added to this block's mergeset
        for &b in &mergeset_blues {
            if !reachability.is_dag_ancestor_of(b, candidate) {
                anticone_blues.insert(b);
            }
        }

        // 2. Trace back the selected parent chain to find blue blocks in candidate's anticone
        let mut current = selected_parent;
        while current != Hash::default() && !reachability.is_dag_ancestor_of(current, candidate) {
            anticone_blues.insert(current);
            if let Ok(blues) = store.get_mergeset_blues(current) {
                for &b in blues.iter() {
                    if !reachability.is_dag_ancestor_of(b, candidate) {
                        anticone_blues.insert(b);
                    }
                }
            }
            current = store.get_selected_parent(current).unwrap_or_default();
        }

        // Check K-cluster conditions:
        // (a) Candidate anticone with blues must not exceed K
        let candidate_valid = anticone_blues.len() <= k as usize;

        // (b) For every blue block in candidate's anticone, its blue anticone size must not exceed K
        let existing_blues_valid = anticone_blues
            .iter()
            .all(|&b| blues_anticone_sizes.get(&b).copied().unwrap_or(0) < k);

        if candidate_valid && existing_blues_valid {
            // Elect candidate as BLUE
            mergeset_blues.push(candidate);
            blues_anticone_sizes.insert(candidate, anticone_blues.len() as KType);

            for &b in &anticone_blues {
                *blues_anticone_sizes.entry(b).or_insert(0) += 1;
            }
        } else {
            // Elect candidate as RED
            mergeset_reds.push(candidate);
        }
    }

    (mergeset_blues, mergeset_reds, blues_anticone_sizes)
}
