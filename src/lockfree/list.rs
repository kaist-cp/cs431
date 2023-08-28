//! Lock-free singly linked list.

use core::cmp::Ordering::{Equal, Greater, Less};
use core::mem;
use core::sync::atomic::Ordering;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

/// Linked list node.
#[derive(Debug)]
pub struct Node<K, V> {
    /// Mark: tag(), Tag: not needed
    next: Atomic<Node<K, V>>,
    key: K,
    value: V,
}

/// Sorted singly linked list.
///
/// Use-after-free will be caused when an unprotected guard is used, as the lifetime of returned
/// elements are linked to that of the guard in the same way a `Shared<'g,T>` is.
#[derive(Debug)]
pub struct List<K, V> {
    head: Atomic<Node<K, V>>,
}

impl<K, V> Default for List<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for List<K, V> {
    fn drop(&mut self) {
        let mut curr = mem::take(&mut self.head);
        // SAFETY: since we have `&mut self`, any references from `lookup()` must have finished.
        // Hence, we have sole ownership of `self` and its `Node`s.
        while let Some(curr_ref) = unsafe { curr.try_into_owned() } {
            curr = curr_ref.into_box().next;
        }
    }
}

/// Linked list cursor.
#[derive(Debug, Copy)]
pub struct Cursor<'g, K, V> {
    prev: &'g Atomic<Node<K, V>>,
    // Tag of `curr` should always be zero so when `curr` is stored in a `prev`, we don't store a
    // marked pointer and cause cleanup to fail.
    curr: Shared<'g, Node<K, V>>,
}

// Manual implementation as deriving `Clone` leads to unnecessary trait bounds.
impl<'g, K, V> Clone for Cursor<'g, K, V> {
    fn clone(&self) -> Self {
        Self {
            prev: self.prev,
            curr: self.curr,
        }
    }
}

impl<K, V> Node<K, V> {
    /// Creates a new node.
    pub fn new(key: K, value: V) -> Self {
        Self {
            next: Atomic::null(),
            key,
            value,
        }
    }

    /// Extracts the inner value.
    pub fn into_value(self) -> V {
        self.value
    }
}

impl<'g, K, V> Cursor<'g, K, V>
where
    K: Ord,
{
    /// Creates a cursor.
    pub fn new(prev: &'g Atomic<Node<K, V>>, curr: Shared<'g, Node<K, V>>) -> Self {
        Self {
            prev,
            curr: curr.with_tag(0),
        }
    }

    /// Returns the current node.
    pub fn curr(&self) -> Shared<'g, Node<K, V>> {
        self.curr
    }

    /// Clean up a chain of logically removed nodes in each traversal.
    #[inline]
    pub fn find_harris(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        // Finding phase
        // - cursor.curr: first unmarked node w/ key >= search key (4)
        // - cursor.prev: the ref of .next in previous unmarked node (1 -> 2)
        // 1 -> 2 -x-> 3 -x-> 4 -> 5 -> ∅  (search key: 4)
        let mut prev_next = self.curr;
        let found = loop {
            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                break false;
            };
            let next = curr_node.next.load(Ordering::Acquire, guard);

            // - finding stage is done if cursor.curr advancement stops
            // - advance cursor.curr if (.next is marked) || (cursor.curr < key)
            // - stop cursor.curr if (not marked) && (cursor.curr >= key)
            // - advance cursor.prev if not marked

            if next.tag() != 0 {
                // We add a 0 tag here so that `self.curr`s tag is always 0.
                self.curr = next.with_tag(0);
                continue;
            }

            match curr_node.key.cmp(key) {
                Less => {
                    self.curr = next;
                    self.prev = &curr_node.next;
                    prev_next = next;
                }
                Equal => break true,
                Greater => break false,
            }
        };

        // If prev and curr WERE adjacent, no need to clean up
        if prev_next == self.curr {
            return Ok(found);
        }

        // cleanup marked nodes between prev and curr
        self.prev
            .compare_exchange(
                prev_next,
                self.curr,
                Ordering::Release,
                Ordering::Relaxed,
                guard,
            )
            .map_err(|_| ())?;

        // defer_destroy from cursor.prev.load() to cursor.curr (exclusive)
        let mut node = prev_next;
        while node.with_tag(0) != self.curr {
            // SAFETY: All nodes in the unlinked chain are not null.
            let next = unsafe { node.deref() }.next.load(Ordering::Relaxed, guard);
            // SAFETY: we unlinked the chain with above CAS.
            unsafe { guard.defer_destroy(node) };
            node = next;
        }

        Ok(found)
    }

    /// Clean up a single logically removed node in each traversal.
    #[inline]
    pub fn find_harris_michael(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        loop {
            debug_assert_eq!(self.curr.tag(), 0);

            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                return Ok(false);
            };
            let mut next = curr_node.next.load(Ordering::Acquire, guard);

            if next.tag() != 0 {
                next = next.with_tag(0);
                self.prev
                    .compare_exchange(self.curr, next, Ordering::Release, Ordering::Relaxed, guard)
                    .map_err(|_| ())?;
                unsafe { guard.defer_destroy(self.curr) };
                self.curr = next;
                continue;
            }

            match curr_node.key.cmp(key) {
                Less => {
                    self.prev = &curr_node.next;
                    self.curr = next;
                }
                Equal => return Ok(true),
                Greater => return Ok(false),
            }
        }
    }

    /// Gotta go fast. Doesn't fail.
    #[inline]
    pub fn find_harris_herlihy_shavit(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        Ok(loop {
            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                break false;
            };
            match curr_node.key.cmp(key) {
                Less => {
                    self.curr = curr_node.next.load(Ordering::Acquire, guard);
                    // NOTE: unnecessary (this function is expected to be used only for `lookup`)
                    self.prev = &curr_node.next;
                    continue;
                }
                Equal => break curr_node.next.load(Ordering::Relaxed, guard).tag() == 0,
                Greater => break false,
            }
        })
    }

    /// Lookups the value.
    #[inline]
    pub fn lookup(&self) -> Option<&'g V> {
        unsafe { self.curr.as_ref() }.map(|n| &n.value)
    }

    /// Inserts a value.
    #[inline]
    pub fn insert(
        &mut self,
        node: Owned<Node<K, V>>,
        guard: &'g Guard,
    ) -> Result<(), Owned<Node<K, V>>> {
        node.next.store(self.curr, Ordering::Relaxed);
        match self.prev.compare_exchange(
            self.curr,
            node,
            Ordering::Release,
            Ordering::Relaxed,
            guard,
        ) {
            Ok(node) => {
                self.curr = node;
                Ok(())
            }
            Err(e) => Err(e.new),
        }
    }

    /// Deletes the current node.
    #[inline]
    pub fn delete(self, guard: &'g Guard) -> Result<&'g V, ()> {
        // SAFETY: curr was found, hence cannot be null.
        let curr_node = unsafe { self.curr.deref() };

        // Release: to release current view of the deleting thread on this mark.
        // Acquire: to ensure that if the latter CAS succeeds, then the thread that reads `next` through `prev` will be safe.
        let next = curr_node.next.fetch_or(1, Ordering::AcqRel, guard);
        if next.tag() == 1 {
            return Err(());
        }

        if self
            .prev
            .compare_exchange(self.curr, next, Ordering::Release, Ordering::Relaxed, guard)
            .is_ok()
        {
            // SAFETY: we are unlinker of curr. As the lifetime of the guard extends to the return
            // value of the function, later access of curr_node is ok.
            unsafe { guard.defer_destroy(self.curr) };
        }

        Ok(&curr_node.value)
    }
}

impl<K, V> List<K, V>
where
    K: Ord,
{
    /// Creates a new list.
    pub fn new() -> Self {
        List {
            head: Atomic::null(),
        }
    }

    /// Creates the head cursor.
    #[inline]
    pub fn head<'g>(&'g self, guard: &'g Guard) -> Cursor<'g, K, V> {
        Cursor::new(&self.head, self.head.load(Ordering::Acquire, guard))
    }

    /// Finds a key using the given find strategy.
    #[inline]
    fn find<'g, F>(&'g self, key: &K, find: &F, guard: &'g Guard) -> (bool, Cursor<'g, K, V>)
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let mut cursor = self.head(guard);
            if let Ok(r) = find(&mut cursor, key, guard) {
                return (r, cursor);
            }
        }
    }

    #[inline]
    fn lookup<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<&'g V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let (found, cursor) = self.find(key, &find, guard);
        if found {
            cursor.lookup()
        } else {
            None
        }
    }

    #[inline]
    fn insert<'g, F>(&'g self, key: K, value: V, find: F, guard: &'g Guard) -> bool
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let mut node = Owned::new(Node::new(key, value));
        loop {
            let (found, mut cursor) = self.find(&node.key, &find, guard);
            if found {
                return false;
            }

            match cursor.insert(node, guard) {
                Err(n) => node = n,
                Ok(()) => return true,
            }
        }
    }

    #[inline]
    fn delete<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<&'g V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let (found, cursor) = self.find(key, &find, guard);
            if !found {
                return None;
            }

            match cursor.delete(guard) {
                Err(()) => continue,
                Ok(value) => return Some(value),
            }
        }
    }

    /// Omitted
    pub fn harris_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_insert<'g>(&'g self, key: K, value: V, guard: &'g Guard) -> bool {
        self.insert(key, value, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_delete<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.delete(key, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_michael_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_michael_insert(&self, key: K, value: V, guard: &Guard) -> bool {
        self.insert(key, value, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_michael_delete<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.delete(key, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris_herlihy_shavit, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_insert(&self, key: K, value: V, guard: &Guard) -> bool {
        self.insert(key, value, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_delete<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.delete(key, Cursor::find_harris_michael, guard)
    }
}
