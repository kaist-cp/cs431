//! Testing utilities for set types.

use core::fmt::Debug;
use core::hash::Hash;

use crossbeam_epoch::Guard;

use super::map;
use crate::test::RandGen;
use crate::{ConcurrentMap, ConcurrentSet};

// A set can be seen as a map with value `()`. Thus, we can reuse the tests for maps.
impl<T, S: ConcurrentSet<T>> ConcurrentMap<T, ()> for S {
    fn lookup<'a>(&'a self, key: &T, _guard: &'a Guard) -> Option<&'a ()> {
        if self.contains(key) {
            Some(&())
        } else {
            None
        }
    }

    fn insert(&self, key: T, _value: (), _guard: &Guard) -> Result<(), ()> {
        if self.insert(key) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn delete<'a>(&'a self, key: &T, _guard: &'a Guard) -> Result<&'a (), ()> {
        if self.remove(key) {
            Ok(&())
        } else {
            Err(())
        }
    }
}

/// See `map::stress_sequential`.
pub fn stress_sequential<T: Debug + Clone + Eq + Hash + RandGen, S: Default + ConcurrentSet<T>>(
    steps: usize,
) {
    map::stress_sequential::<T, (), S>(steps);
}

/// See `map::stress_concurrent`.
pub fn stress_concurrent<T: Debug + Eq + RandGen, S: Default + Sync + ConcurrentSet<T>>(
    threads: usize,
    steps: usize,
) {
    map::stress_concurrent::<T, (), S>(threads, steps);
}

/// See `map::log_concurrent`.
pub fn log_concurrent<
    T: Clone + Debug + Eq + Hash + RandGen + Send,
    S: Default + Sync + ConcurrentSet<T>,
>(
    threads: usize,
    steps: usize,
) {
    map::log_concurrent::<T, (), S>(threads, steps);
}
