#[cfg(test)]
mod tests {
    use crate::model::services::reachability::ReachabilityService;
    use crate::model::stores::ghostdag::{GhostdagData, GhostdagStore};
    use crate::model::stores::reachability::ReachabilityStore;
    use crate::model::stores::relations::RelationsStore;
    use crate::processes::ghostdag::protocol::GhostdagManager;
    use jio_consensus_core::blockhash::BlockHash;
    use jio_hashes::Hash;
    use jio_math::Uint192;
    use std::sync::Arc;

    fn setup_ghostdag_manager(k: u64) -> (GhostdagManager, GhostdagStore, RelationsStore, ReachabilityService) {
        let ghostdag_store = GhostdagStore::new();
        let relations = RelationsStore::new();
        let reachability_store = ReachabilityStore::new();
        let reachability = ReachabilityService::new(relations.clone(), reachability_store);
        let manager = GhostdagManager::new(k, ghostdag_store.clone(), relations.clone(), reachability.clone());
        (manager, ghostdag_store, relations, reachability)
    }

    #[test]
    fn test_diamond_dag_coloring() {
        let (manager, ghostdag_store, relations, reachability) = setup_ghostdag_manager(18);

        let genesis = Hash::from_bytes([1u8; 32]);
        let block_b = Hash::from_bytes([2u8; 32]);
        let block_c = Hash::from_bytes([3u8; 32]);
        let block_d = Hash::from_bytes([4u8; 32]);

        // 1. Genesis
        reachability.init_genesis(genesis);
        ghostdag_store.insert(
            genesis,
            Arc::new(GhostdagData::new(0, Uint192::ZERO, BlockHash::default(), vec![], vec![])),
        );

        // 2. Block B (child of Genesis)
        relations.insert(block_b, vec![genesis]);
        reachability.add_block(block_b, genesis);
        let gd_b = manager.ghostdag(&[genesis]);
        ghostdag_store.insert(block_b, Arc::new(gd_b));

        // 3. Block C (child of Genesis)
        relations.insert(block_c, vec![genesis]);
        reachability.add_block(block_c, genesis);
        let gd_c = manager.ghostdag(&[genesis]);
        ghostdag_store.insert(block_c, Arc::new(gd_c));

        // 4. Block D (merges B and C)
        relations.insert(block_d, vec![block_b, block_c]);
        reachability.add_block(block_d, block_b);
        let gd_d = manager.ghostdag(&[block_b, block_c]);

        assert_eq!(gd_d.selected_parent, block_b.min(block_c)); // Tie-breaking selects lower hash
        let non_selected = if gd_d.selected_parent == block_b { block_c } else { block_b };
        assert!(gd_d.mergeset_blues.contains(&non_selected));
        assert!(gd_d.mergeset_reds.is_empty());
        assert_eq!(gd_d.blue_score, 3); // Genesis(0) -> Selected(1) -> NonSelected(+1) -> D(+1) = 3
    }

    #[test]
    fn test_k_cluster_anticone_limit_enforcement() {
        // Set K = 2: Each blue block can have at most K=2 blue blocks in its anticone
        let (manager, ghostdag_store, relations, reachability) = setup_ghostdag_manager(2);

        let genesis = Hash::from_bytes([0u8; 32]);
        reachability.init_genesis(genesis);
        ghostdag_store.insert(
            genesis,
            Arc::new(GhostdagData::new(0, Uint192::ZERO, BlockHash::default(), vec![], vec![])),
        );

        // Create 5 parallel blocks off Genesis: P1, P2, P3, P4, P5
        let mut parallel_blocks = Vec::new();
        for i in 1..=5u8 {
            let p = Hash::from_bytes([i; 32]);
            relations.insert(p, vec![genesis]);
            reachability.add_block(p, genesis);
            let gd_p = manager.ghostdag(&[genesis]);
            ghostdag_store.insert(p, Arc::new(gd_p));
            parallel_blocks.push(p);
        }

        // Merge block M merging all 5 parallel blocks
        let merge_block = Hash::from_bytes([99u8; 32]);
        relations.insert(merge_block, parallel_blocks.clone());
        reachability.add_block(merge_block, parallel_blocks[0]);

        let gd_merge = manager.ghostdag(&parallel_blocks);

        // 1 selected parent (P1)
        // In candidate set {P2, P3, P4, P5}:
        // With K=2, at most 3 blocks (P2, P3, P4) can form a mutually anticone blue cluster.
        // The 4th candidate (P5) must be colored RED!
        assert_eq!(gd_merge.mergeset_blues.len(), 3);
        assert_eq!(gd_merge.mergeset_reds.len(), 1);
        assert_eq!(gd_merge.mergeset_blues.len() + gd_merge.mergeset_reds.len(), 4); // 5 parents - 1 selected_parent = 4
    }
}
