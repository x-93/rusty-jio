pub trait MemSizeEstimator {
    fn estimate_mem_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
