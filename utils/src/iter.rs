//! Iterator helper extensions.

pub trait IteratorExtensions: Iterator {
    fn filter_some<T>(self) -> impl Iterator<Item = T>
    where
        Self: Sized + Iterator<Item = Option<T>>,
    {
        self.flatten()
    }
}

impl<I: Iterator> IteratorExtensions for I {}
