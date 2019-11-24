//! Concurrent binary search tree protected with optimistic lock coupling.
//!
//! - From Bronson, Casper, Chafi, Olukotun. A Practical Concurrent Binary Search Tree. PPoPP 2010
//! (https://stanford-ppl.github.io/website/papers/ppopp207-bronson.pdf)
//!
//! - We implement partially external relaxed tree (section 3) with a few simplifications.

use core::cmp;
use core::mem::{self, ManuallyDrop};
use core::sync::atomic::Ordering;
use crossbeam_epoch::{unprotected, Atomic, Guard, Owned, Shared};
use lock::seqlock::{ReadGuard, SeqLock};

mod base;

use crate::map::ConcurrentMap;
pub use base::Bst;
use base::{AtomicRW, Cursor, Dir, Node, NodeInner};

impl<'g, K: Ord, V> Cursor<'g, K, V> {
    /// Discards the current node.
    fn pop(&mut self) -> Result<(), ()> {
        unimplemented!()
    }

    /// Pushs a new node as the current one.
    ///
    /// Returns `Err(())` if the existing current node's guard is invalidated.
    fn push(
        &mut self,
        current: Shared<'g, Node<K, V>>,
        guard: ReadGuard<'g, NodeInner<K, V>>,
        dir: Dir,
    ) -> Result<(), ()> {
        unimplemented!()
    }

    /// Finds the given `key` from the current cursor (`self`).
    ///
    /// - Returns `Ordering::Less` or `Ordering::Greater` if the key should be inserted from the
    ///   left or right (resp.)  child of the resulting cursor.
    /// - Returns `Ordering::Equal` if the key was found.
    fn find(&mut self, key: &K, guard: &'g Guard) -> cmp::Ordering {
        unimplemented!()
    }

    // Recursively tries to unlink `self.current` if it's vacant and at least one of children is
    // null.
    fn cleanup(&mut self, guard: &Guard) {
        unimplemented!()
    }
}

impl<K: Ord, V> ConcurrentMap<K, V> for Bst<K, V>
where
    K: Clone,
    Option<V>: AtomicRW,
{
    /// Inserts the given `value` at the given `key`.
    ///
    /// - Returns `Ok(())` if `value` is inserted.
    /// - Returns `Err(value)` for the given `value` if `key` is already occupied.
    fn insert<'a>(&'a self, key: &'a K, value: V, guard: &'a Guard) -> Result<(), V> {
        unimplemented!()
    }

    /// Deletes the given `key`.
    ///
    /// - Returns `Ok(value)` if `value` was deleted from `key`.
    /// - Returns `Err(())` if `key` was vacant.
    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()> {
        unimplemented!()
    }

    /// Looks up the given `key` and calls `f` for the found `value`.
    fn lookup<'a, F, R>(&'a self, key: &'a K, guard: &'a Guard, f: F) -> R
    where
        F: FnOnce(Option<&V>) -> R,
    {
        unimplemented!()
    }
}

impl<K: Ord, V> Drop for Bst<K, V> {
    fn drop(&mut self) {
        unimplemented!()
    }
}
