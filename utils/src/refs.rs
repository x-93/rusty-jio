//! Reference utilities and wrapper helpers.

use std::sync::Arc;

pub trait RefExtensions<T> {
    fn into_arc(self) -> Arc<T>;
}

impl<T> RefExtensions<T> for T {
    fn into_arc(self) -> Arc<T> {
        Arc::new(self)
    }
}
