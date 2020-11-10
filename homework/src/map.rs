use core::marker::PhantomData;
use crossbeam_epoch::Guard;
use lock::{Lock, RawLock};
use rand::{distributions::Alphanumeric, rngs::ThreadRng, Rng};

/// Types that has random generator
pub trait RandGen {
    /// Randomly generates a value.
    fn rand_gen(rng: &mut ThreadRng) -> Self;
}

const KEY_MAX_LENGTH: usize = 4;

impl RandGen for String {
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        let length = rng.gen::<usize>() % KEY_MAX_LENGTH;
        rng.sample_iter(&Alphanumeric).take(length).collect()
    }
}

impl RandGen for usize {
    /// pick only 16 bits, MSB=0
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        const MASK: usize = 0x4444444444444444usize;
        rng.gen::<usize>() & MASK
    }
}

impl RandGen for u32 {
    /// pick only 16 bits
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        const MASK: u32 = 0x66666666u32;
        rng.gen::<u32>() & MASK
    }
}

/// Trait for a sequential key-value map.
pub trait SequentialMap<K: ?Sized, V> {
    /// Lookups a key.
    fn lookup<'a>(&'a self, key: &'a K) -> Option<&'a V>;

    /// Inserts a key-value pair.
    fn insert<'a>(&'a mut self, key: &'a K, value: V) -> Result<&'a mut V, (&'a mut V, V)>;

    /// Inserts a key.
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

    /// Inserts a key.
    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()>;
}

/// Trait for a nonblocking key-value map.
pub trait NonblockingMap<K: ?Sized, V> {
    /// Lookups the given key to get the reference to its value.
    fn lookup<'a>(&'a self, key: &K, guard: &'a Guard) -> Option<&'a V>;

    /// Inserts a key-value pair.
    fn insert(&self, key: &K, value: V, guard: &Guard) -> Result<(), V>;

    /// Deletes the given key and its value.
    fn delete<'a>(&'a self, key: &K, guard: &'a Guard) -> Result<&'a V, ()>;
}

/// Converts str sequential map into string sequential map
#[derive(Default, Debug)]
pub struct StrStringMap<V, M: SequentialMap<str, V>> {
    inner: M,
    _marker: PhantomData<V>,
}

impl<V, M: SequentialMap<str, V>> SequentialMap<String, V> for StrStringMap<V, M> {
    fn lookup<'a>(&'a self, key: &'a String) -> Option<&'a V> {
        self.inner.lookup(key)
    }

    fn insert<'a>(&'a mut self, key: &'a String, value: V) -> Result<&'a mut V, (&'a mut V, V)> {
        self.inner.insert(key, value)
    }

    fn delete(&mut self, key: &String) -> Result<V, ()> {
        self.inner.delete(key)
    }
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
        self.lock()
            .insert(key, value)
            .map(|_| ())
            .map_err(|(_, v)| v)
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
