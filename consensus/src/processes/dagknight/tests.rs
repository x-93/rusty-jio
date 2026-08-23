use super::appendable_segment_tree_api::{AppendableSegmentTreeApi, Bucket};
use super::appendable_segment_tree_impl::AppendableSegmentTree;
use super::rank_search::RankSearcher;
use super::umc_cascade::{BlockWithWork, CascadeMaintainer};
use crate::model::services::reachability::ReachabilityService;
use crate::model::stores::dagknight::DagknightKey;
use jio_consensus_core::BlueWorkType;
use jio_hashes::Hash;

#[test]
fn test_dagknight_key_encoding_and_equality() {
    let root = Hash::from_u64_4(1, 2, 3, 4);
    let pov = Hash::from_u64_4(5, 6, 7, 8);
    let key1 = DagknightKey::new(root, pov, 18, false);
    let key2 = DagknightKey::new(root, pov, 18, false);
    let key3 = DagknightKey::new(root, pov, 18, true);
    let key4 = DagknightKey::new(root, pov, 19, false);

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
    assert_ne!(key1, key4);
    assert_eq!(key1.as_ref().len(), 67);
}

#[test]
fn test_appendable_segment_tree_operations() {
    let mut tree: AppendableSegmentTree<u32, i64> = AppendableSegmentTree::new();
    assert_eq!(tree.len(), 0);
    assert!(!tree.has_positive_below_zero());
    assert!(!tree.has_negative_above_zero());

    // Append leaves with initial scores
    tree.append_leaf(1, 10);
    tree.append_leaf(2, 20);
    tree.append_leaf(3, -5);
    tree.append_leaf(4, 15);

    assert_eq!(tree.len(), 4);
    assert_eq!(tree.score_of(&1), Some(10));
    assert_eq!(tree.score_of(&2), Some(20));
    assert_eq!(tree.score_of(&3), Some(-5));
    assert_eq!(tree.score_of(&4), Some(15));

    // Prefix subtract: subtract 15 from prefix of length 2 (leaves 1 and 2)
    tree.prefix_add(2, -15);
    assert_eq!(tree.score_of(&1), Some(-5)); // dropped below 0
    assert_eq!(tree.score_of(&2), Some(5));

    // Now leaf 1 was in positive bucket and dropped below zero
    assert!(tree.has_positive_below_zero());
    let popped = tree.pop_positive_below_zero();
    assert_eq!(popped, Some((1, -5)));
    assert!(!tree.has_positive_below_zero());

    // Range add: add 10 to leaf 3 (range 2..3)
    tree.range_add(2..3, 10);
    assert_eq!(tree.score_of(&3), Some(5)); // -5 + 10 = 5, rose above 0

    assert!(tree.has_negative_above_zero());
    let popped_neg = tree.pop_negative_above_zero();
    assert_eq!(popped_neg, Some((3, 5)));
    assert!(!tree.has_negative_above_zero());
}

#[test]
fn test_appendable_segment_tree_growth() {
    let mut tree: AppendableSegmentTree<usize, i64> = AppendableSegmentTree::with_initial_capacity(4);

    for i in 0..32 {
        tree.append_leaf(i, i as i64 * 10);
    }
    assert_eq!(tree.len(), 32);

    for i in 0..32 {
        assert_eq!(tree.score_of(&i), Some(i as i64 * 10));
    }

    // Prefix update across all 32 leaves
    tree.prefix_add(32, -50);
    for i in 0..32 {
        assert_eq!(tree.score_of(&i), Some(i as i64 * 10 - 50));
    }
}

#[test]
fn test_rank_searcher() {
    // Condition: valid for k >= 7
    let result = RankSearcher::search(|k| {
        if k >= 7 {
            Some(format!("valid_at_{}", k))
        } else {
            None
        }
    });

    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.k, 7);
    assert_eq!(res.result, "valid_at_7");

    // Condition: valid for k >= 0 (immediate)
    let result_zero = RankSearcher::search(|k| Some(k * 2));
    assert_eq!(result_zero.unwrap().k, 0);

    // Condition: valid for k >= 100
    let result_100 = RankSearcher::search(|k| if k >= 100 { Some(true) } else { None });
    assert_eq!(result_100.unwrap().k, 100);
}

struct MockReachability;

impl ReachabilityService for MockReachability {
    fn is_chain_ancestor_of(&self, _descendant: Hash, _ancestor: Hash) -> bool {
        false
    }
    fn is_dag_ancestor_of_result(&self, ancestor: Hash, descendant: Hash) -> crate::processes::reachability::Result<bool> {
        Ok(self.is_dag_ancestor_of(ancestor, descendant))
    }
    fn is_dag_ancestor_of(&self, ancestor: Hash, descendant: Hash) -> bool {
        ancestor.as_bytes()[0] < descendant.as_bytes()[0]
    }
    fn is_dag_ancestor_of_any(&self, ancestor: Hash, descendants: &mut impl Iterator<Item = Hash>) -> bool {
        descendants.any(|d| self.is_dag_ancestor_of(ancestor, d))
    }
    fn is_any_dag_ancestor(&self, list: &mut impl Iterator<Item = Hash>, queried: Hash) -> bool {
        list.any(|a| self.is_dag_ancestor_of(a, queried))
    }
    fn is_any_dag_ancestor_result(&self, list: &mut impl Iterator<Item = Hash>, queried: Hash) -> crate::processes::reachability::Result<bool> {
        Ok(self.is_any_dag_ancestor(list, queried))
    }
    fn get_next_chain_ancestor(&self, _descendant: Hash, _ancestor: Hash) -> Hash {
        Hash::default()
    }
    fn get_chain_parent(&self, _this: Hash) -> Hash {
        Hash::default()
    }
    fn has_reachability_data(&self, _this: Hash) -> bool {
        true
    }
}

#[test]
fn test_cascade_maintainer() {
    let cg = BlockWithWork::new(Hash::from_u64_4(1, 0, 0, 0), BlueWorkType::from(100u64));
    let mut cascade = CascadeMaintainer::new(cg, 16); // k = 16 => sqrt(16) = 4 => deficit = 400

    let mock_reachability = MockReachability;

    let b1 = BlockWithWork::new(Hash::from_u64_4(2, 0, 0, 0), BlueWorkType::from(50u64));
    let b2 = BlockWithWork::new(Hash::from_u64_4(3, 0, 0, 0), BlueWorkType::from(50u64));
    let r1 = BlockWithWork::new(Hash::from_u64_4(4, 0, 0, 0), BlueWorkType::from(20u64));

    cascade.add_blue(b1, &mock_reachability);
    cascade.add_blue(b2, &mock_reachability);
    cascade.add_red(r1, &mock_reachability);

    assert!(cascade.is_valid());
}
