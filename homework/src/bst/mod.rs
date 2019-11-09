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

    fn find(&mut self, key: &K, guard: &'g Guard) -> cmp::Ordering {
        unimplemented!()
    }

    fn cleanup(&mut self, guard: &Guard) {
        unimplemented!()
    }
}

impl<K: Ord, V> ConcurrentMap<K, V> for Bst<K, V>
where
    K: Clone,
    Option<V>: AtomicRW,
{
    fn insert<'a>(&'a self, key: &'a K, value: V, guard: &'a Guard) -> Result<(), V> {
        unimplemented!()
    }

    fn delete(&self, key: &K, guard: &Guard) -> Result<V, ()> {
        unimplemented!()
    }

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
