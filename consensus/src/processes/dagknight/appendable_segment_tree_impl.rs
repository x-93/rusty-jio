use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, AddAssign, Range, Sub},
};

use super::appendable_segment_tree_api::{
    bucket_for_score, AppendableSegmentTreeApi, Bucket, ScoreZero, DEFAULT_INITIAL_CAPACITY,
};

type LeafPosition = usize;
type NodeIndex = usize;

const ROOT_NODE: NodeIndex = 1;

fn left_child(node: NodeIndex) -> NodeIndex {
    node * 2
}

fn right_child(node: NodeIndex) -> NodeIndex {
    node * 2 + 1
}

fn parent(node: NodeIndex) -> NodeIndex {
    debug_assert!(node > ROOT_NODE, "root node has no parent");
    node / 2
}

fn ranges_are_disjoint(first: &Range<LeafPosition>, second: &Range<LeafPosition>) -> bool {
    first.end <= second.start || second.end <= first.start
}

fn range_fully_contains(outer: &Range<LeafPosition>, inner: &Range<LeafPosition>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn split_range(range: &Range<LeafPosition>) -> (Range<LeafPosition>, Range<LeafPosition>) {
    debug_assert!(range.len() > 1, "cannot split a leaf range");
    let midpoint = range.start + range.len() / 2;
    (range.start..midpoint, midpoint..range.end)
}

#[derive(Clone, Copy, Debug)]
struct ScoreCandidate<T, S> {
    score: S,
    leaf: T,
}

#[derive(Clone, Debug)]
struct BucketExtrema<T, S> {
    min_positive: Option<ScoreCandidate<T, S>>,
    max_negative: Option<ScoreCandidate<T, S>>,
    pending_delta: S,
}

impl<T, S> BucketExtrema<T, S>
where
    T: Copy,
    S: Copy + PartialOrd + AddAssign + ScoreZero,
{
    fn empty() -> Self {
        Self { min_positive: None, max_negative: None, pending_delta: S::zero() }
    }

    fn create_leaf_with_score(leaf: T, score: S) -> Self {
        let candidate = Some(ScoreCandidate { score, leaf });
        match bucket_for_score(score) {
            Bucket::Positive => Self { min_positive: candidate, max_negative: None, pending_delta: S::zero() },
            Bucket::Negative => Self { min_positive: None, max_negative: candidate, pending_delta: S::zero() },
        }
    }

    fn merge(left_child: &Self, right_child: &Self) -> Self {
        Self {
            min_positive: minimum_candidate(left_child.min_positive, right_child.min_positive),
            max_negative: maximum_candidate(left_child.max_negative, right_child.max_negative),
            pending_delta: S::zero(),
        }
    }

    fn apply_delta(&mut self, delta: S) {
        if let Some(candidate) = self.min_positive.as_mut() {
            candidate.score += delta;
        }
        if let Some(candidate) = self.max_negative.as_mut() {
            candidate.score += delta;
        }
        self.pending_delta += delta;
    }
}

fn minimum_candidate<T: Copy, S: Copy + PartialOrd>(
    left_candidate: Option<ScoreCandidate<T, S>>,
    right_candidate: Option<ScoreCandidate<T, S>>,
) -> Option<ScoreCandidate<T, S>> {
    match (left_candidate, right_candidate) {
        (Some(left_candidate), Some(right_candidate)) => {
            Some(if left_candidate.score <= right_candidate.score { left_candidate } else { right_candidate })
        }
        (candidate @ Some(_), None) | (None, candidate @ Some(_)) => candidate,
        (None, None) => None,
    }
}

fn maximum_candidate<T: Copy, S: Copy + PartialOrd>(
    left_candidate: Option<ScoreCandidate<T, S>>,
    right_candidate: Option<ScoreCandidate<T, S>>,
) -> Option<ScoreCandidate<T, S>> {
    match (left_candidate, right_candidate) {
        (Some(left_candidate), Some(right_candidate)) => {
            Some(if left_candidate.score >= right_candidate.score { left_candidate } else { right_candidate })
        }
        (candidate @ Some(_), None) | (None, candidate @ Some(_)) => candidate,
        (None, None) => None,
    }
}

pub struct AppendableSegmentTree<T, S = i64> {
    len: usize,
    leaf_capacity: usize,
    position_by_leaf: HashMap<T, LeafPosition>,
    leaf_by_position: Vec<T>,
    nodes: Vec<BucketExtrema<T, S>>,
}

impl<T, S> AppendableSegmentTree<T, S>
where
    T: Copy + Eq + Hash,
    S: Copy + PartialOrd + AddAssign + ScoreZero + Add<Output = S> + Sub<Output = S>,
{
    pub fn new() -> Self {
        Self::with_initial_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    pub fn with_initial_capacity(initial_capacity: usize) -> Self {
        let leaf_capacity = initial_capacity.max(1).next_power_of_two();
        let total_nodes = leaf_capacity * 2;
        Self {
            len: 0,
            leaf_capacity,
            position_by_leaf: HashMap::new(),
            leaf_by_position: Vec::new(),
            nodes: vec![BucketExtrema::empty(); total_nodes],
        }
    }

    fn push_down(&mut self, node: NodeIndex) {
        let delta = self.nodes[node].pending_delta;
        if !delta.is_zero() {
            let left = left_child(node);
            let right = right_child(node);
            if left < self.nodes.len() {
                self.nodes[left].apply_delta(delta);
            }
            if right < self.nodes.len() {
                self.nodes[right].apply_delta(delta);
            }
            self.nodes[node].pending_delta = S::zero();
        }
    }

    fn ensure_capacity_for_next_leaf(&mut self) {
        if self.len == self.leaf_capacity {
            let old_capacity = self.leaf_capacity;
            let new_capacity = old_capacity * 2;
            let mut new_tree = Self::with_initial_capacity(new_capacity);

            // Re-insert all current leaves with their latest resolved scores
            for i in 0..self.len {
                let leaf = self.leaf_by_position[i];
                let score = self.get_leaf_score(i);
                new_tree.append_leaf(leaf, score);
            }
            *self = new_tree;
        }
    }

    fn get_leaf_score(&self, position: LeafPosition) -> S {
        let mut node = ROOT_NODE;
        let mut range = 0..self.leaf_capacity;
        let mut accumulated_delta = S::zero();

        while range.len() > 1 {
            accumulated_delta += self.nodes[node].pending_delta;
            let (left_range, right_range) = split_range(&range);
            if left_range.contains(&position) {
                node = left_child(node);
                range = left_range;
            } else {
                node = right_child(node);
                range = right_range;
            }
        }
        let leaf_extrema = &self.nodes[node];
        let base_score = if let Some(pos) = leaf_extrema.min_positive {
            pos.score
        } else if let Some(neg) = leaf_extrema.max_negative {
            neg.score
        } else {
            S::zero()
        };
        base_score + accumulated_delta
    }

    fn update_range(&mut self, node: NodeIndex, node_range: Range<LeafPosition>, target_range: &Range<LeafPosition>, delta: S) {
        if ranges_are_disjoint(&node_range, target_range) {
            return;
        }

        if range_fully_contains(target_range, &node_range) {
            self.nodes[node].apply_delta(delta);
            return;
        }

        self.push_down(node);
        let (left_range, right_range) = split_range(&node_range);
        let left = left_child(node);
        let right = right_child(node);

        self.update_range(left, left_range, target_range, delta);
        self.update_range(right, right_range, target_range, delta);

        let left_extrema = self.nodes[left].clone();
        let right_extrema = self.nodes[right].clone();
        self.nodes[node] = BucketExtrema::merge(&left_extrema, &right_extrema);
    }
}

impl<T, S> Default for AppendableSegmentTree<T, S>
where
    T: Copy + Eq + Hash,
    S: Copy + PartialOrd + AddAssign + ScoreZero + Add<Output = S> + Sub<Output = S>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, S> AppendableSegmentTreeApi<T, S> for AppendableSegmentTree<T, S>
where
    T: Copy + Eq + Hash,
    S: Copy + PartialOrd + AddAssign + ScoreZero + Add<Output = S> + Sub<Output = S>,
{
    fn len(&self) -> usize {
        self.len
    }

    fn append_leaf(&mut self, leaf: T, score: S) {
        self.ensure_capacity_for_next_leaf();
        let position = self.len;
        self.position_by_leaf.insert(leaf, position);
        self.leaf_by_position.push(leaf);
        self.len += 1;

        let leaf_index = self.leaf_capacity + position;
        self.nodes[leaf_index] = BucketExtrema::create_leaf_with_score(leaf, score);

        let mut current = parent(leaf_index);
        while current >= ROOT_NODE {
            let left = &self.nodes[left_child(current)].clone();
            let right = &self.nodes[right_child(current)].clone();
            self.nodes[current] = BucketExtrema::merge(left, right);
            if current == ROOT_NODE {
                break;
            }
            current = parent(current);
        }
    }

    fn prefix_add(&mut self, prefix_len: usize, delta: S) {
        if prefix_len == 0 || self.len == 0 || delta.is_zero() {
            return;
        }
        let actual_len = prefix_len.min(self.len);
        self.range_add(0..actual_len, delta);
    }

    fn range_add(&mut self, range: Range<usize>, delta: S) {
        if range.is_empty() || self.len == 0 || delta.is_zero() {
            return;
        }
        let clamped_range = range.start.min(self.len)..range.end.min(self.len);
        if clamped_range.is_empty() {
            return;
        }
        let full_range = 0..self.leaf_capacity;
        self.update_range(ROOT_NODE, full_range, &clamped_range, delta);
    }

    fn has_positive_below_zero(&self) -> bool {
        if self.len == 0 {
            return false;
        }
        if let Some(candidate) = self.nodes[ROOT_NODE].min_positive {
            candidate.score < S::zero()
        } else {
            false
        }
    }

    fn has_negative_above_zero(&self) -> bool {
        if self.len == 0 {
            return false;
        }
        if let Some(candidate) = self.nodes[ROOT_NODE].max_negative {
            candidate.score >= S::zero()
        } else {
            false
        }
    }

    fn pop_positive_below_zero(&mut self) -> Option<(T, S)> {
        if !self.has_positive_below_zero() {
            return None;
        }

        let mut node = ROOT_NODE;
        let mut range = 0..self.leaf_capacity;

        while range.len() > 1 {
            self.push_down(node);
            let (left_range, right_range) = split_range(&range);
            let left = left_child(node);
            let right = right_child(node);

            if let Some(pos) = self.nodes[left].min_positive {
                if pos.score < S::zero() {
                    node = left;
                    range = left_range;
                    continue;
                }
            }
            node = right;
            range = right_range;
        }

        // At leaf node: flip to negative
        let candidate = self.nodes[node].min_positive.take().expect("leaf must have min_positive");
        let flipped = ScoreCandidate { score: candidate.score, leaf: candidate.leaf };
        self.nodes[node].max_negative = Some(flipped);

        let mut current = parent(node);
        while current >= ROOT_NODE {
            let left = &self.nodes[left_child(current)].clone();
            let right = &self.nodes[right_child(current)].clone();
            self.nodes[current] = BucketExtrema::merge(left, right);
            if current == ROOT_NODE {
                break;
            }
            current = parent(current);
        }

        Some((candidate.leaf, candidate.score))
    }

    fn pop_negative_above_zero(&mut self) -> Option<(T, S)> {
        if !self.has_negative_above_zero() {
            return None;
        }

        let mut node = ROOT_NODE;
        let mut range = 0..self.leaf_capacity;

        while range.len() > 1 {
            self.push_down(node);
            let (left_range, right_range) = split_range(&range);
            let left = left_child(node);
            let right = right_child(node);

            if let Some(neg) = self.nodes[left].max_negative {
                if neg.score >= S::zero() {
                    node = left;
                    range = left_range;
                    continue;
                }
            }
            node = right;
            range = right_range;
        }

        // At leaf node: flip to positive
        let candidate = self.nodes[node].max_negative.take().expect("leaf must have max_negative");
        let flipped = ScoreCandidate { score: candidate.score, leaf: candidate.leaf };
        self.nodes[node].min_positive = Some(flipped);

        let mut current = parent(node);
        while current >= ROOT_NODE {
            let left = &self.nodes[left_child(current)].clone();
            let right = &self.nodes[right_child(current)].clone();
            self.nodes[current] = BucketExtrema::merge(left, right);
            if current == ROOT_NODE {
                break;
            }
            current = parent(current);
        }

        Some((candidate.leaf, candidate.score))
    }

    fn score_of(&self, leaf: &T) -> Option<S> {
        let &pos = self.position_by_leaf.get(leaf)?;
        Some(self.get_leaf_score(pos))
    }
}
