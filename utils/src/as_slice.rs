//! Trait for types that can be borrowed as a slice.

pub trait AsSlice<T> {
    fn as_slice(&self) -> &[T];
}

impl<T> AsSlice<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> AsSlice<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T> AsSlice<T> for [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
