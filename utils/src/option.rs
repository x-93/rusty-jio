pub trait OptionExtension<T> {
    fn is_some_and_matches<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool;
}

impl<T> OptionExtension<T> for Option<T> {
    fn is_some_and_matches<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool,
    {
        match self {
            Some(x) => f(x),
            None => false,
        }
    }
}
