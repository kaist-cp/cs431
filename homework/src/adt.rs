use core::marker::PhantomData;
use crossbeam_epoch::Guard;
use cs431::lock::{Lock, RawLock};
use rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};

/// Trait for a sequential key-value map.
pub trait SequentialMap<K: ?Sized, V> {
    /// Lookups a key.
    fn lookup<'a>(&'a self, key: &'a K) -> Option<&'a V>;

    /// Inserts a key-value pair.
    // fn insert<'a>(&'a mut self, key: &'a K, value: V) -> Result<&'a mut V, (&'a mut V, V)>;
    fn insert<'a>(&'a mut self, key: &'a K, value: V) -> Result<(), V>;

    /// Deletes a key, returning the value.
    fn delete(&mut self, key: &K) -> Result<V, ()>;
}

/// Trait for a concurrent key-value map.
pub trait ConcurrentMap<K: ?Sized, V> {
    /// Lookups a key.
    fn lookup<'a, F, R>(&'a self, key: &'a K, guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R;

    /// Inserts a key-value pair.
    fn insert<'a>(&'a self, key: &'a K, value: V, guard: &'a Guard) -> Result<(), V>;

    /// Deletes the given key and returns its value.
    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()>;
}

/// Trait for a nonblocking key-value map.
pub trait NonblockingMap<K: ?Sized, V> {
    /// Lookups the given key to get the reference to its value.
    fn lookup<'a>(&'a self, key: &K, guard: &'a Guard) -> Option<&'a V>;

    /// Inserts a key-value pair.
    fn insert(&self, key: &K, value: V, guard: &Guard) -> Result<(), V>;

    /// Deletes the given key and returns a reference to its value.
    ///
    /// Unlike stack or queue's pop that can return `Option<V>`, since a `delete`d
    /// value may also be `lookup`ed, we can only return a reference, not full ownership.
    fn delete<'a>(&'a self, key: &K, guard: &'a Guard) -> Result<&'a V, ()>;
}

impl<K: ?Sized, V, L: RawLock, M> ConcurrentMap<K, V> for Lock<L, M>
where
    M: SequentialMap<K, V>,
{
    fn lookup<'a, F, R>(&'a self, key: &'a K, _guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R,
    {
        f(self.lock().lookup(key))
    }

    fn insert<'a>(&'a self, key: &'a K, value: V, _guard: &'a Guard) -> Result<(), V> {
        self.lock().insert(key, value)
    }

    fn delete(&self, key: &K, _guard: &Guard) -> Result<V, ()> {
        self.lock().delete(key)
    }
}

/// Converts nonblocking map into concurrent map
#[derive(Default, Debug)]
pub struct NonblockingConcurrentMap<K: ?Sized, V: Clone, M: NonblockingMap<K, V>> {
    inner: M,
    _marker: PhantomData<(Box<K>, V)>,
}

impl<K: ?Sized, V: Clone, M: NonblockingMap<K, V>> ConcurrentMap<K, V>
    for NonblockingConcurrentMap<K, V, M>
{
    fn lookup<'a, F, R>(&'a self, key: &'a K, guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R,
    {
        f(self.inner.lookup(key, guard))
    }

    fn insert<'a>(&'a self, key: &'a K, value: V, guard: &'a Guard) -> Result<(), V> {
        self.inner.insert(key, value, guard)
    }

    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()> {
        self.inner.delete(key, guard).map(|v| v.clone())
    }
}

/// Trait for a concurrent set
pub trait ConcurrentSet<T> {
    /// Returns `true` iff the set contains the value.
    fn contains(&self, value: &T) -> bool;

    /// Adds the value to the set. Returns whether the value was newly inserted.
    fn insert(&self, value: T) -> bool;

    /// Removes the value from the set. Returns whether the value was present in the set.
    fn remove(&self, value: &T) -> bool;
}
