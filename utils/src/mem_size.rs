//! Memory footprint tracking trait for accurate cache sizing and memory bounds.

use std::{
    collections::{HashMap, HashSet},
    mem::size_of,
    sync::Arc,
};

/// Trait for measuring the heap and stack memory size of an object in bytes.
pub trait MemSize {
    fn mem_size(&self) -> usize;
}

/// Trait for estimating memory bytes (used across consensus structures).
pub trait MemSizeEstimator {
    fn estimate_mem_bytes(&self) -> usize;
}

macro_rules! impl_mem_size_primitive {
    ($($t:ty),*) => {
        $(
            impl MemSize for $t {
                #[inline(always)]
                fn mem_size(&self) -> usize {
                    size_of::<$t>()
                }
            }
        )*
    };
}

impl_mem_size_primitive!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    bool,
    f32,
    f64,
    ()
);

impl<T: MemSize, const N: usize> MemSize for [T; N] {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.iter().map(|item| item.mem_size() - size_of::<T>()).sum::<usize>()
    }
}

impl<T: MemSize> MemSize for Vec<T> {
    fn mem_size(&self) -> usize {
        let heap_elements: usize = self.iter().map(|item| item.mem_size()).sum();
        size_of::<Self>() + self.capacity() * size_of::<T>() + heap_elements - (self.len() * size_of::<T>())
    }
}

impl MemSize for String {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.capacity()
    }
}

impl MemSize for &str {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.len()
    }
}

impl<T: MemSize> MemSize for Box<T> {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.as_ref().mem_size()
    }
}

impl<T: MemSize> MemSize for Arc<T> {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.as_ref().mem_size()
    }
}

impl<T: MemSize> MemSize for Option<T> {
    fn mem_size(&self) -> usize {
        size_of::<Self>() + self.as_ref().map_or(0, |item| item.mem_size() - size_of::<T>())
    }
}

impl<K: MemSize, V: MemSize> MemSize for HashMap<K, V> {
    fn mem_size(&self) -> usize {
        size_of::<Self>()
            + (self.capacity() * (size_of::<K>() + size_of::<V>() + size_of::<usize>()))
            + self
                .iter()
                .map(|(k, v)| (k.mem_size() - size_of::<K>()) + (v.mem_size() - size_of::<V>()))
                .sum::<usize>()
    }
}

impl<T: MemSize> MemSize for HashSet<T> {
    fn mem_size(&self) -> usize {
        size_of::<Self>()
            + (self.capacity() * (size_of::<T>() + size_of::<usize>()))
            + self.iter().map(|item| item.mem_size() - size_of::<T>()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mem_size() {
        assert_eq!(42u64.mem_size(), 8);
        assert_eq!(true.mem_size(), 1);
    }

    #[test]
    fn test_vec_mem_size() {
        let v: Vec<u64> = vec![1, 2, 3, 4];
        assert!(v.mem_size() >= size_of::<Vec<u64>>() + 4 * 8);
    }

    #[test]
    fn test_string_mem_size() {
        let s = String::from("hello world");
        assert!(s.mem_size() >= size_of::<String>() + 11);
    }
}
