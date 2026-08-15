use super::Result;
use crate::model::stores::reachability::ReachabilityStoreReader;
use jio_consensus_core::blockhash;
use jio_hashes::Hash;

pub fn is_chain_ancestor_of(
    store: &(impl ReachabilityStoreReader + ?Sized),
    this: Hash,
    queried: Hash,
) -> Result<bool> {
    if this == queried || this == blockhash::ORIGIN {
        return Ok(true);
    }
    if queried == blockhash::ORIGIN {
        return Ok(false);
    }
    let this_interval = match store.get_interval(this) {
        Some(interval) => interval,
        None => return Ok(false),
    };
    let queried_interval = match store.get_interval(queried) {
        Some(interval) => interval,
        None => return Ok(false),
    };
    Ok(this_interval.contains(&queried_interval))
}

pub fn is_dag_ancestor_of(
    store: &(impl ReachabilityStoreReader + ?Sized),
    this: Hash,
    queried: Hash,
) -> Result<bool> {
    if this == queried || this == blockhash::ORIGIN {
        return Ok(true);
    }
    if queried == blockhash::ORIGIN {
        return Ok(false);
    }
    // Fast path: tree interval containment
    if is_chain_ancestor_of(store, this, queried)? {
        return Ok(true);
    }

    // Traversal path up the tree
    let mut queue = std::collections::VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back(queried);
    visited.insert(queried);

    while let Some(current) = queue.pop_front() {
        if let Some(parent) = store.get_parent(current) {
            if parent == this {
                return Ok(true);
            }
            if is_chain_ancestor_of(store, this, parent)? {
                return Ok(true);
            }
            if parent != blockhash::ORIGIN && visited.insert(parent) {
                queue.push_back(parent);
            }
        }
    }

    Ok(false)
}

pub fn get_next_chain_ancestor(
    store: &(impl ReachabilityStoreReader + ?Sized),
    descendant: Hash,
    ancestor: Hash,
) -> Result<Hash> {
    if descendant == ancestor {
        return Ok(ancestor);
    }
    let descendant_interval = store
        .get_interval(descendant)
        .ok_or_else(|| super::ReachabilityError::MissingInterval(descendant))?;

    if let Some(children) = store.get_children(ancestor) {
        for &child in children.iter() {
            if child == descendant {
                return Ok(child);
            }
            if let Some(child_interval) = store.get_interval(child) {
                if child_interval.contains(&descendant_interval) {
                    return Ok(child);
                }
            }
        }
    }

    Err(super::ReachabilityError::NotChainAncestor(ancestor, descendant))
}
