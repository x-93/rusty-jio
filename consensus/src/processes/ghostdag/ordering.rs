use crate::model::stores::ghostdag::GhostdagStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_math::Uint192;

/// Sorts blocks by GHOSTDAG topological order: (blue_work, blue_score, hash)
pub fn sort_blocks_topological(blocks: &mut [BlockHash], ghostdag_store: &GhostdagStore) {
    blocks.sort_by(|a, b| {
        let work_a = ghostdag_store.get_blue_work(a).unwrap_or(Uint192::ZERO);
        let work_b = ghostdag_store.get_blue_work(b).unwrap_or(Uint192::ZERO);

        work_a.cmp(&work_b).then_with(|| {
            let score_a = ghostdag_store.get_blue_score(a).unwrap_or(0);
            let score_b = ghostdag_store.get_blue_score(b).unwrap_or(0);
            score_a.cmp(&score_b).then_with(|| a.cmp(b))
        })
    });
}

pub fn sort_blocks(blocks: &mut [BlockHash]) {
    blocks.sort();
}
