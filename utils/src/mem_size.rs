pub trait MemSizeEstimator {
    fn estimate_mem_size(&self) -> usize {
        0
    }
    fn estimate_mem_bytes(&self) -> usize {
        self.estimate_mem_size()
    }
}

impl<T: MemSizeEstimator> MemSizeEstimator for Vec<T> {
    fn estimate_mem_size(&self) -> usize {
        let heap: usize = self.iter().map(|item| item.estimate_mem_size()).sum();
        std::mem::size_of::<Vec<T>>() + (self.capacity() * std::mem::size_of::<T>()) + heap
    }
}

impl MemSizeEstimator for u8 {
    fn estimate_mem_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl MemSizeEstimator for u64 {
    fn estimate_mem_size(&self) -> usize {
        std::mem::size_of::<u64>()
    }
}
