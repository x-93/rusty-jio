use std::collections::HashMap;

use jio_consensus_core::{BlueWorkType, KType};
use jio_hashes::Hash;
use jio_math::int::SignedInteger;

use crate::model::services::reachability::ReachabilityService;
use super::appendable_segment_tree_api::{bucket_for_score, AppendableSegmentTreeApi, Bucket};
use super::appendable_segment_tree_impl::AppendableSegmentTree;

type SignedWork = SignedInteger<BlueWorkType>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockWithWork {
    pub hash: Hash,
    pub work: BlueWorkType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockColor {
    BLUE,
    RED,
}

impl BlockWithWork {
    pub fn new(hash: Hash, work: BlueWorkType) -> Self {
        Self { hash, work }
    }
}

pub struct CascadeMaintainer {
    blues_chains_decomposition: Vec<Vec<Hash>>,
    chains_score_trees: Vec<AppendableSegmentTree<BlockWithWork, SignedWork>>,
    blk_mapping_to_chains: HashMap<Hash, (usize, usize)>, // hash -> (chain_idx, pos_in_chain)
    deficit_work: BlueWorkType,
    blue_work: BlueWorkType,
    red_work: BlueWorkType,
    negative_blue_work: BlueWorkType,
    flip_count: u64,
}

impl CascadeMaintainer {
    pub fn new(conflict_genesis: BlockWithWork, k: KType) -> Self {
        // sqrt(k) as u64
        let k_isqrt = (k as f64).sqrt() as u64;
        let deficit_work = conflict_genesis.work * k_isqrt;
        Self {
            blues_chains_decomposition: Vec::new(),
            chains_score_trees: Vec::new(),
            blk_mapping_to_chains: HashMap::new(),
            deficit_work,
            blue_work: BlueWorkType::from(0u64),
            red_work: BlueWorkType::from(0u64),
            negative_blue_work: BlueWorkType::from(0u64),
            flip_count: 0,
        }
    }

    fn find_extendable_chain(&self, hash: Hash, reachability: &impl ReachabilityService) -> Option<usize> {
        for (i, chain) in self.blues_chains_decomposition.iter().enumerate() {
            if let Some(&tip) = chain.last() {
                if reachability.is_dag_ancestor_of(tip, hash) {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn add_blue(&mut self, block: BlockWithWork, reachability: &impl ReachabilityService) {
        self.blue_work += block.work;
        let initial_score = SignedWork::from(self.deficit_work);

        let chain_id = if let Some(cid) = self.find_extendable_chain(block.hash, reachability) {
            cid
        } else {
            let new_cid = self.blues_chains_decomposition.len();
            self.blues_chains_decomposition.push(Vec::new());
            self.chains_score_trees.push(AppendableSegmentTree::new());
            new_cid
        };

        let pos_in_chain = self.blues_chains_decomposition[chain_id].len();
        self.blues_chains_decomposition[chain_id].push(block.hash);
        self.blk_mapping_to_chains.insert(block.hash, (chain_id, pos_in_chain));
        self.chains_score_trees[chain_id].append_leaf(block, initial_score);

        // Propagate block.work to all blue ancestors of this block
        let work_signed = SignedWork::from(block.work);
        for c_idx in 0..self.blues_chains_decomposition.len() {
            if c_idx == chain_id {
                // All blocks before pos_in_chain in the same chain are direct ancestors
                self.chains_score_trees[c_idx].prefix_add(pos_in_chain, work_signed);
            } else {
                for (p_idx, &b_hash) in self.blues_chains_decomposition[c_idx].iter().enumerate() {
                    if reachability.is_dag_ancestor_of(b_hash, block.hash) {
                        self.chains_score_trees[c_idx].range_add(p_idx..(p_idx + 1), work_signed);
                    }
                }
            }
        }

        self.stabilize(reachability);
    }

    pub fn add_red(&mut self, block: BlockWithWork, reachability: &impl ReachabilityService) {
        self.red_work += block.work;
        let sub_delta = SignedWork::from(0u64) - SignedWork::from(block.work);

        for c_idx in 0..self.blues_chains_decomposition.len() {
            for (p_idx, &b_hash) in self.blues_chains_decomposition[c_idx].iter().enumerate() {
                if reachability.is_dag_ancestor_of(b_hash, block.hash) {
                    self.chains_score_trees[c_idx].range_add(p_idx..(p_idx + 1), sub_delta);
                }
            }
        }

        self.stabilize(reachability);
    }

    fn stabilize(&mut self, reachability: &impl ReachabilityService) {
        loop {
            let mut flipped = false;

            for c_idx in 0..self.chains_score_trees.len() {
                while let Some((leaf, _score)) = self.chains_score_trees[c_idx].pop_positive_below_zero() {
                    flipped = true;
                    self.flip_count += 1;
                    self.negative_blue_work += leaf.work;
                    // Delta to ancestors is -2 * work
                    let delta = SignedWork::from(0u64) - SignedWork::from(leaf.work * 2u64);
                    self.propagate_to_ancestors(leaf.hash, delta, reachability);
                }

                while let Some((leaf, _score)) = self.chains_score_trees[c_idx].pop_negative_above_zero() {
                    flipped = true;
                    self.flip_count += 1;
                    self.negative_blue_work -= leaf.work;
                    // Delta to ancestors is +2 * work
                    let delta = SignedWork::from(leaf.work * 2u64);
                    self.propagate_to_ancestors(leaf.hash, delta, reachability);
                }
            }

            if !flipped {
                break;
            }
        }
    }

    fn propagate_to_ancestors(&mut self, hash: Hash, delta: SignedWork, reachability: &impl ReachabilityService) {
        let (chain_id, pos_in_chain) = self.blk_mapping_to_chains[&hash];
        for c_idx in 0..self.blues_chains_decomposition.len() {
            if c_idx == chain_id {
                self.chains_score_trees[c_idx].prefix_add(pos_in_chain, delta);
            } else {
                for (p_idx, &b_hash) in self.blues_chains_decomposition[c_idx].iter().enumerate() {
                    if reachability.is_dag_ancestor_of(b_hash, hash) {
                        self.chains_score_trees[c_idx].range_add(p_idx..(p_idx + 1), delta);
                    }
                }
            }
        }
    }

    pub fn virtual_score(&self) -> SignedWork {
        let signed_blue = SignedWork::from(self.blue_work) - SignedWork::from(self.negative_blue_work * 2u64);
        signed_blue + SignedWork::from(self.deficit_work) - SignedWork::from(self.red_work)
    }

    pub fn is_valid(&self) -> bool {
        self.virtual_score() >= SignedWork::from(0u64)
    }

    pub fn flip_count(&self) -> u64 {
        self.flip_count
    }
}
