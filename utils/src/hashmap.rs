//! HashMap utilities and extensions.

use std::collections::HashMap;
use std::hash::Hash;

pub trait HashMapExtensions<K, V> {
    fn get_or_insert_default(&mut self, key: K) -> &mut V
    where
        V: Default;
}

impl<K: Eq + Hash, V> HashMapExtensions<K, V> for HashMap<K, V> {
    fn get_or_insert_default(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        self.entry(key).or_default()
    }
}
