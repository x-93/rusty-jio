use jio_consensus_core::KType;
use jio_core::debug;

#[derive(Clone, Debug)]
pub struct RankSearchResult<T> {
    pub k: KType,
    pub result: T,
}

pub struct RankSearcher;

impl RankSearcher {
    /// K-searching logic:
    /// 1. Search for an upper bound using powers of 2
    ///    1.1 For each unsuccessful step along the way, move the lower bound k up
    /// 2. Binary search between lower bound k and upper bound k
    pub fn search<T, F>(mut evaluate: F) -> Option<RankSearchResult<T>>
    where
        F: FnMut(KType) -> Option<T>,
    {
        let mut lower_k: KType = 0;
        let mut upper_k: KType = 0;
        let mut upper_result: Option<T> = None;
        let mut step: KType = 1;

        // Step 1: Exponential search for upper bound
        loop {
            debug!("DAGKNIGHT: Finding upper bound k = {}", upper_k);
            if let Some(r) = evaluate(upper_k) {
                debug!("DAGKNIGHT: Found valid upper bound at k = {}", upper_k);
                upper_result = Some(r);
                break;
            }

            lower_k = upper_k.saturating_add(1);
            if upper_k == KType::MAX {
                break;
            }

            upper_k = upper_k.saturating_add(step);
            step = step.saturating_mul(2);
        }

        let mut best_result = upper_result?;
        let mut best_k = upper_k;

        // Step 2: Binary search between lower_k and upper_k
        if upper_k > 0 {
            let mut low = lower_k;
            let mut high = upper_k.saturating_sub(1);

            while low <= high {
                let mid = low + (high - low) / 2;
                debug!("DAGKNIGHT: Binary search checking mid k = {}", mid);
                if let Some(r) = evaluate(mid) {
                    best_k = mid;
                    best_result = r;
                    if mid == 0 {
                        break;
                    }
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            }
        }

        Some(RankSearchResult { k: best_k, result: best_result })
    }
}
