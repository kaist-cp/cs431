use crossbeam_epoch::Guard;
use lock::{Lock, RawLock};

/// Trait for a sequential key-value map.
pub trait SequentialMap<K: ?Sized, V> {
    /// Inserts a key-value pair.
    fn insert<'a>(&'a mut self, key: &'a K, value: V) -> Result<&'a mut V, (&'a mut V, V)>;

    /// Inserts a key.
    fn delete(&mut self, key: &K) -> Result<V, ()>;

    /// Lookups a key.
    fn lookup<'a>(&'a self, key: &'a K) -> Option<&'a V>;
}

/// Trait for a concurrent key-value map.
pub trait ConcurrentMap<K: ?Sized, V> {
    /// Inserts a key-value pair.
    fn insert<'a>(&'a self, key: &'a K, value: V, guard: &'a Guard) -> Result<(), V>;

    /// Inserts a key.
    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()>;

    /// Lookups a key.
    fn lookup<'a, F, R>(&'a self, key: &'a K, guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R;
}

impl<K: ?Sized, V, L: RawLock, M> ConcurrentMap<K, V> for Lock<L, M>
where
    M: SequentialMap<K, V>,
{
    fn insert<'a>(&'a self, key: &'a K, value: V, _guard: &'a Guard) -> Result<(), V> {
        self.lock()
            .insert(key, value)
            .map(|_| ())
            .map_err(|(_, v)| v)
    }

    fn delete(&self, key: &K, _guard: &Guard) -> Result<V, ()> {
        self.lock().delete(key)
    }

    fn lookup<'a, F, R>(&'a self, key: &'a K, _guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R,
    {
        f(self.lock().lookup(key))
    }
}
