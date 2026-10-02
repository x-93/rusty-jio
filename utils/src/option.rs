//! Option extensions.

pub trait OptionExtensions<T> {
    fn replace_if(&mut self, condition: bool, value: T) -> Option<T>;
}

impl<T> OptionExtensions<T> for Option<T> {
    fn replace_if(&mut self, condition: bool, value: T) -> Option<T> {
        if condition {
            self.replace(value)
        } else {
            None
        }
    }
}
