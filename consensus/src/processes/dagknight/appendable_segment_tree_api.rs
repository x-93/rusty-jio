use std::ops::Range;

pub(super) const DEFAULT_INITIAL_CAPACITY: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bucket {
    Positive,
    Negative,
}

/// Trait representing zero value for generic scores.
pub trait ScoreZero: Copy + PartialOrd {
    fn zero() -> Self;
    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl ScoreZero for i64 {
    fn zero() -> Self {
        0
    }
}

impl<T: Copy + PartialOrd + Default> ScoreZero for jio_math::int::SignedInteger<T> {
    fn zero() -> Self {
        jio_math::int::SignedInteger::from(T::default())
    }
}

/// Assigns non-negative scores to the positive bucket and negative scores to the negative bucket.
pub fn bucket_for_score<S: ScoreZero>(score: S) -> Bucket {
    if score >= S::zero() {
        Bucket::Positive
    } else {
        Bucket::Negative
    }
}

/// Public API for the appendable segment tree used by UMC cascade.
pub trait AppendableSegmentTreeApi<T, S> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn append_leaf(&mut self, leaf: T, score: S);
    fn prefix_add(&mut self, prefix_len: usize, delta: S);
    fn range_add(&mut self, range: Range<usize>, delta: S);
    fn has_positive_below_zero(&self) -> bool;
    fn has_negative_above_zero(&self) -> bool;
    fn pop_positive_below_zero(&mut self) -> Option<(T, S)>;
    fn pop_negative_above_zero(&mut self) -> Option<(T, S)>;
    fn score_of(&self, leaf: &T) -> Option<S>;
}
