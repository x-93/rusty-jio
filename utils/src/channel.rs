//! Bounded and unbounded channel wrappers with error types.

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};

pub fn mpsc_bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    bounded(cap)
}

pub fn mpsc_unbounded<T>() -> (Sender<T>, Receiver<T>) {
    unbounded()
}
