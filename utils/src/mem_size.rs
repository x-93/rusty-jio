use std::mem::size_of;
use std::sync::Arc;

/// A trait for estimating heap and stack memory size of data structures in bytes.
pub trait MemSizeEstimator {
    fn estimate_mem_bytes(&self) -> usize;
}

macro_rules! impl_primitive_mem_size {
    ($($t:ty),*) => {
        $(
            impl MemSizeEstimator for $t {
                #[inline(always)]
                fn estimate_mem_bytes(&self) -> usize {
                    size_of::<Self>()
                }
            }
        )*
    };
}

impl_primitive_mem_size!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char
);

impl<T: MemSizeEstimator, const N: usize> MemSizeEstimator for [T; N] {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .iter()
                .map(MemSizeEstimator::estimate_mem_bytes)
                .sum::<usize>()
            - (size_of::<T>() * N)
    }
}

impl<T: MemSizeEstimator> MemSizeEstimator for Vec<T> {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self.capacity() * size_of::<T>()
            + self
                .iter()
                .map(MemSizeEstimator::estimate_mem_bytes)
                .sum::<usize>()
            - (size_of::<T>() * self.len())
    }
}

impl<T: MemSizeEstimator> MemSizeEstimator for Arc<T> {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.as_ref().estimate_mem_bytes()
    }
}

impl<T: MemSizeEstimator> MemSizeEstimator for Option<T> {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .as_ref()
                .map(MemSizeEstimator::estimate_mem_bytes)
                .unwrap_or(0)
    }
}

impl<T: MemSizeEstimator> MemSizeEstimator for Box<T> {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.as_ref().estimate_mem_bytes()
    }
}

impl MemSizeEstimator for String {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.capacity()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> MemSizeEstimator for smallvec::SmallVec<A>
where
    A::Item: MemSizeEstimator,
{
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + if self.spilled() {
                self.capacity() * size_of::<A::Item>()
            } else {
                0
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(42u64.estimate_mem_bytes(), 8);
        assert_eq!(true.estimate_mem_bytes(), 1);
    }

    #[test]
    fn test_vec() {
        let v = vec![1u32, 2, 3];
        assert!(v.estimate_mem_bytes() >= size_of::<Vec<u32>>() + 3 * size_of::<u32>());
    }
}
