use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct MinNonNan<T>(pub T);

impl<T: PartialEq> PartialEq for MinNonNan<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for MinNonNan<T> {}

impl<T: PartialOrd> PartialOrd for MinNonNan<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<T: Ord> Ord for MinNonNan<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

pub type MinHeap<T> = BinaryHeap<MinNonNan<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_heap() {
        let mut heap = MinHeap::new();
        heap.push(MinNonNan(10));
        heap.push(MinNonNan(5));
        heap.push(MinNonNan(20));

        assert_eq!(heap.pop().unwrap().0, 5);
        assert_eq!(heap.pop().unwrap().0, 10);
        assert_eq!(heap.pop().unwrap().0, 20);
    }
}
