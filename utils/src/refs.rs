use std::ops::Deref;

pub struct Ref<'a, T> {
    inner: &'a T,
}

impl<'a, T> Ref<'a, T> {
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> AsRef<T> for Ref<'a, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}
