pub trait VecExtensions<T> {
    fn extend_with<I: IntoIterator<Item = T>>(&mut self, iter: I);
}

impl<T> VecExtensions<T> for Vec<T> {
    fn extend_with<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.extend(iter);
    }
}
