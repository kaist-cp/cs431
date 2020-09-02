//! Thead-safe key/value cache.

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};

/// Cache that remembers the result for each key.
#[derive(Debug, Default)]
pub struct Cache<K, V> {
    // todo! Build your own cache type.
    inner: ()
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    /// Retrieve the value or insert a new one created by `f`.
    ///
    /// An invocation to this function should not block another invocation with a different key.
    /// For exmaple, if a thread calls `get_or_insert_with(key1, f1)` and another thread calls
    /// `get_or_insert_with(key2, f2)` (`key1≠key2`, `key1,key2∉cache`) concurrently, `f1` and `f2`
    /// should run concurrently.
    ///
    /// On the other hand, since `f` may consume a lot of resource (= money), it's desirable not to
    /// duplicate the work. That is, `f` should be run only once for each key. Specifically, even
    /// for the concurrent invocations of `get_or_insert_with(key, f)`, `f` is called only once.
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
        todo!()
    }
}
