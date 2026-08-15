use std::sync::Arc;

pub trait ArcExtension<T> {
    fn unwrap_or_clone(self) -> T
    where
        T: Clone;
}

impl<T> ArcExtension<T> for Arc<T> {
    fn unwrap_or_clone(self) -> T
    where
        T: Clone,
    {
        Arc::try_unwrap(self).unwrap_or_else(|arc| (*arc).clone())
    }
}
