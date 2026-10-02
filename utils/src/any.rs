//! Utilities for working with `dyn Any`.

use std::any::Any;

pub trait AnyExtension {
    fn is<T: Any>(&self) -> bool;
    fn downcast_ref<T: Any>(&self) -> Option<&T>;
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T>;
}

impl AnyExtension for dyn Any {
    fn is<T: Any>(&self) -> bool {
        self.type_id() == std::any::TypeId::of::<T>()
    }

    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Any as *const T)) }
        } else {
            None
        }
    }

    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Any as *mut T)) }
        } else {
            None
        }
    }
}
