use crossbeam_epoch::{unprotected, Atomic, Guard, Owned, Shared};

use std::cmp::Ordering::{Equal, Greater, Less};
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering;

#[derive(Debug)]
struct Node<K, V> {
    /// Mark: tag(), Tag: not needed
    next: Atomic<Node<K, V>>,
    key: K,
    value: ManuallyDrop<V>,
}

/// Sorted singly linked list.
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
        unsafe {
            let mut curr = self.head.load(Ordering::Relaxed, unprotected());

            while !curr.is_null() {
                let curr_ref = curr.deref_mut();
                let next = curr_ref.next.load(Ordering::Relaxed, unprotected());
                if next.tag() == 0 {
                    ManuallyDrop::drop(&mut curr_ref.value);
                }
                drop(curr.into_owned());
                curr = next;
            }
        }
    }
}

struct Cursor<'g, K, V> {
    prev: &'g Atomic<Node<K, V>>,
    curr: Shared<'g, Node<K, V>>,
}

impl<'g, K, V> Cursor<'g, K, V>
where
    K: Ord,
{
    /// Clean up a chain of logically removed nodes in each traversal.
    #[inline]
    fn find_harris(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        // Finding phase
        // - cursor.curr: first unmarked node w/ key >= search key (4)
        // - cursor.prev: the ref of .next in previous unmarked node (1 -> 2)
        // 1 -> 2 -x-> 3 -x-> 4 -> 5 -> ∅  (search key: 4)
        let mut prev_next = self.curr;
        let found = loop {
            let curr_node = some_or!(unsafe { self.curr.as_ref() }, break false);
            let mut next = curr_node.next.load(Ordering::Acquire, guard);

            // - finding stage is done if cursor.curr advancement stops
            // - advance cursor.curr if (.next is marked) || (cursor.curr < key)
            // - stop cursor.curr if (not marked) && (cursor.curr >= key)
            // - advance cursor.prev if not marked
            match (curr_node.key.cmp(key), next.tag()) {
                (Less, tag) => {
                    self.curr = next.with_tag(0);
                    if tag == 0 {
                        self.prev = &curr_node.next;
                        prev_next = next;
                    }
                }
                (eq, 0) => {
                    next = curr_node.next.load(Ordering::Acquire, guard);
                    if next.tag() == 0 {
                        break eq == Equal;
                    } else {
                        return Err(());
                    }
                }
                (_, _) => self.curr = next.with_tag(0),
            }
        };

        // If prev and curr WERE adjacent, no need to clean up
        if prev_next == self.curr {
            return Ok(found);
        }

        // cleanup marked nodes between prev and curr
        if self
            .prev
            .compare_and_set(prev_next, self.curr, Ordering::AcqRel, guard)
            .is_err()
        {
            return Err(());
        }

        // defer_destroy from cursor.prev.load() to cursor.curr (exclusive)
        let mut node = prev_next;
        loop {
            if node.with_tag(0) == self.curr {
                return Ok(found);
            }
            let node_ref = unsafe { node.as_ref().unwrap() };
            let next = node_ref.next.load(Ordering::Acquire, guard);
            unsafe {
                guard.defer_destroy(node);
            }
            node = next;
        }
    }

    /// Clean up a single logically removed node in each traversal.
    #[inline]
    fn find_harris_michael(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        loop {
            debug_assert_eq!(self.curr.tag(), 0);

            let curr_node = some_or!(unsafe { self.curr.as_ref() }, return Ok(false));
            let mut next = curr_node.next.load(Ordering::Acquire, guard);

            if next.tag() == 0 {
                match curr_node.key.cmp(key) {
                    Less => self.prev = &curr_node.next,
                    Equal => return Ok(true),
                    Greater => return Ok(false),
                }
            } else {
                next = next.with_tag(0);
                match self
                    .prev
                    .compare_and_set(self.curr, next, Ordering::AcqRel, guard)
                {
                    Err(_) => return Err(()),
                    Ok(_) => unsafe { guard.defer_destroy(self.curr) },
                }
            }
            self.curr = next;
        }
    }

    /// Gotta go fast. Doesn't fail.
    #[inline]
    fn find_harris_herlihy_shavit(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        Ok(loop {
            let curr_node = some_or!(unsafe { self.curr.as_ref() }, break false);
            match curr_node.key.cmp(key) {
                Less => {
                    self.curr = curr_node.next.load(Ordering::Acquire, guard);
                    self.prev = &curr_node.next; // NOTE: not needed
                    continue;
                }
                _ => break curr_node.next.load(Ordering::Acquire, guard).tag() == 0,
            }
        })
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

    /// Finds a key using the given find strategy.
    #[inline]
    fn find<'g, F>(&'g self, key: &K, find: &F, guard: &'g Guard) -> (bool, Cursor<'g, K, V>)
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let mut cursor = Cursor {
                prev: &self.head,
                curr: self.head.load(Ordering::Acquire, guard),
            };
            if let Ok(r) = find(&mut cursor, key, guard) {
                return (r, cursor);
            }
        }
    }

    #[inline]
    fn get<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<&'g V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let (found, cursor) = self.find(key, &find, guard);
        if found {
            unsafe { cursor.curr.as_ref().map(|n| &*n.value) }
        } else {
            None
        }
    }

    #[inline]
    fn insert<'g, F>(&'g self, key: K, value: V, find: F, guard: &'g Guard) -> bool
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let mut node = Owned::new(Node {
            key,
            value: ManuallyDrop::new(value),
            next: Atomic::null(),
        });

        loop {
            let (found, cursor) = self.find(&node.key, &find, guard);
            if found {
                unsafe {
                    ManuallyDrop::drop(&mut node.value);
                }
                return false;
            }

            node.next.store(cursor.curr, Ordering::Relaxed);
            match cursor
                .prev
                .compare_and_set(cursor.curr, node, Ordering::AcqRel, guard)
            {
                Ok(_) => return true,
                Err(e) => node = e.new,
            }
        }
    }

    #[inline]
    fn remove<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let (found, cursor) = self.find(key, &find, guard);
            if !found {
                return None;
            }

            let curr_node = unsafe { cursor.curr.as_ref() }.unwrap();
            let value = unsafe { ptr::read(&curr_node.value) };

            let next = curr_node.next.fetch_or(1, Ordering::AcqRel, guard);
            if next.tag() == 1 {
                continue;
            }

            if cursor
                .prev
                .compare_and_set(cursor.curr, next, Ordering::AcqRel, guard)
                .is_ok()
            {
                unsafe { guard.defer_destroy(cursor.curr) };
            }

            return Some(ManuallyDrop::into_inner(value));
        }
    }

    /// Omitted
    pub fn harris_get<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.get(key, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_insert<'g>(&'g self, key: K, value: V, guard: &'g Guard) -> bool {
        self.insert(key, value, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_remove<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<V> {
        self.remove(key, Cursor::find_harris, guard)
    }

    /// Omitted
    pub fn harris_michael_get<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.get(key, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_michael_insert(&self, key: K, value: V, guard: &Guard) -> bool {
        self.insert(key, value, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_michael_remove(&self, key: &K, guard: &Guard) -> Option<V> {
        self.remove(key, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_get<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.get(key, Cursor::find_harris_herlihy_shavit, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_insert(&self, key: K, value: V, guard: &Guard) -> bool {
        self.insert(key, value, Cursor::find_harris_michael, guard)
    }

    /// Omitted
    pub fn harris_herlihy_shavit_remove(&self, key: &K, guard: &Guard) -> Option<V> {
        self.remove(key, Cursor::find_harris_michael, guard)
    }
}
