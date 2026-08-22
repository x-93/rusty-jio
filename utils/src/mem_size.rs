pub trait MemSizeEstimator {
    fn estimate_mem_size(&self) -> usize {
        self.estimate_mem_bytes()
    }

    fn estimate_mem_bytes(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
