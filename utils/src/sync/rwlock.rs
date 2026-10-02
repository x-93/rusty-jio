//! Parking lot RwLock wrapper.

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct SafeRwLock<T: ?Sized>(RwLock<T>);

impl<T> SafeRwLock<T> {
    pub const fn new(val: T) -> Self {
        Self(RwLock::new(val))
    }

    pub fn read(&self) -> SafeRwLockReadGuard<'_, T> {
        SafeRwLockReadGuard(self.0.read())
    }

    pub fn write(&self) -> SafeRwLockWriteGuard<'_, T> {
        SafeRwLockWriteGuard(self.0.write())
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

pub struct SafeRwLockReadGuard<'a, T: ?Sized>(RwLockReadGuard<'a, T>);

impl<T: ?Sized> Deref for SafeRwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct SafeRwLockWriteGuard<'a, T: ?Sized>(RwLockWriteGuard<'a, T>);

impl<T: ?Sized> Deref for SafeRwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized> DerefMut for SafeRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
