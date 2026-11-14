use crate::model::stores::ghostdag::GhostdagStoreReader;
use jio_hashes::Hash;

/// Sorts the merge set topologically according to GHOSTDAG ordering rules:
/// 1. Ascending by blue work
/// 2. Ascending by blue score
/// 3. Ascending by hash (lexicographical tie-breaking)
pub fn sort_mergeset(store: &impl GhostdagStoreReader, mergeset: &mut [Hash]) {
    mergeset.sort_by(|&a, &b| {
        let (work_a, score_a) = store
            .get(a)
            .map(|d| (d.blue_work, d.blue_score))
            .unwrap_or_default();
        let (work_b, score_b) = store
            .get(b)
            .map(|d| (d.blue_work, d.blue_score))
            .unwrap_or_default();

        work_a
            .cmp(&work_b)
            .then_with(|| score_a.cmp(&score_b))
            .then_with(|| a.cmp(&b))
    });
}
