use crossbeam_epoch::Guard;

/// Trait for a concurrent key-value map.
pub trait ConcurrentMap<K: ?Sized, V> {
    /// Lookups the given key to get the reference to its value.
    fn lookup<'a>(&'a self, key: &K, guard: &'a Guard) -> Option<&'a V>;

    /// Inserts a key-value pair.
    fn insert(&self, key: K, value: V, guard: &Guard) -> Result<(), V>;

    /// Deletes the given key and returns a reference to its value.
    ///
    /// Unlike stack or queue's pop that can return `Option<V>`, since a `delete`d
    /// value may also be `lookup`ed, we can only return a reference, not full ownership.
    fn delete<'a>(&'a self, key: &K, guard: &'a Guard) -> Result<&'a V, ()>;
}

/// Trait for a concurrent set.
pub trait ConcurrentSet<T> {
    /// Returns `true` iff the set contains the value.
    fn contains(&self, value: &T) -> bool;

    /// Adds the value to the set. Returns whether the value was newly inserted.
    fn insert(&self, value: T) -> bool;

    /// Removes the value from the set. Returns whether the value was present in the set.
    fn remove(&self, value: &T) -> bool;
}
