//! Vector helper extensions.

pub trait VecExtensions<T> {
    fn push_if_some(&mut self, item: Option<T>);
}

impl<T> VecExtensions<T> for Vec<T> {
    fn push_if_some(&mut self, item: Option<T>) {
        if let Some(val) = item {
            self.push(val);
        }
    }
}
