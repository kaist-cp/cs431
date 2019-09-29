use crossbeam_epoch::Guard;
use lock::{Lock, RawLock};

/// Trait for a sequential key-value map.
pub trait SequentialMap<V> {
    /// Inserts a key-value pair.
    fn insert<'a>(&'a mut self, key: &'a str, value: V) -> Result<&'a mut V, (&'a mut V, V)>;

    /// Inserts a key.
    fn delete(&mut self, key: &str) -> Result<V, ()>;

    /// Lookups a key.
    fn lookup<'a>(&'a self, key: &'a str) -> Option<&'a V>;
}

/// Trait for a concurrent key-value map.
pub trait ConcurrentMap<V> {
    /// Inserts a key-value pair.
    fn insert<'a>(&'a self, key: &'a str, value: V, guard: &'a Guard) -> Result<(), V>;

    /// Inserts a key.
    fn delete(&self, key: &str, guard: &Guard) -> Result<V, ()>;

    /// Lookups a key.
    fn lookup<'a, F, R>(&'a self, key: &'a str, guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R;
}

impl<V, L: RawLock, M> ConcurrentMap<V> for Lock<L, M>
where
    M: SequentialMap<V>,
{
    fn insert<'a>(&'a self, key: &'a str, value: V, _guard: &'a Guard) -> Result<(), V> {
        self.lock()
            .insert(key, value)
            .map(|_| ())
            .map_err(|(_, v)| v)
    }

    fn delete(&self, key: &str, _guard: &Guard) -> Result<V, ()> {
        self.lock().delete(key)
    }

    fn lookup<'a, F, R>(&'a self, key: &'a str, _guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R,
    {
        f(self.lock().lookup(key))
    }
}
