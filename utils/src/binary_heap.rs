//! Binary heap extensions.

use std::collections::BinaryHeap;

pub trait BinaryHeapExtensions<T> {
    fn drain_filter<F>(&mut self, filter: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool;
}

impl<T: Ord> BinaryHeapExtensions<T> for BinaryHeap<T> {
    fn drain_filter<F>(&mut self, mut filter: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut matching = Vec::new();
        let mut remaining = Vec::new();
        for item in self.drain() {
            if filter(&item) {
                matching.push(item);
            } else {
                remaining.push(item);
            }
        }
        *self = BinaryHeap::from(remaining);
        matching
    }
}
