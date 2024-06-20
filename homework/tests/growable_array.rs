#![feature(cfg_sanitize)]

use core::ops::Deref;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{pin, Guard, Owned, Shared};
use cs431_homework::test::adt::map;
use cs431_homework::{ConcurrentMap, GrowableArray};
use stack::{Node, Stack};

#[derive(Debug)]
struct ArrayMap<V> {
    array: GrowableArray<Node<V>>,
    /// dump everything into a stack and drop them later
    storage: Stack<V>,
}

impl<V> Default for ArrayMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ArrayMap<V> {
    fn new() -> Self {
        Self {
            array: GrowableArray::new(),
            storage: Stack::new(),
        }
    }
}

/// Simple map implementation using the array index as the key.
/// Uses u32 key instead of u64 to limit memory usage and runtime
impl<V> ConcurrentMap<u32, V> for ArrayMap<V> {
    fn lookup<'g>(&self, key: &u32, guard: &'g Guard) -> Option<&'g V> {
        let slot = self.array.get(*key as usize, guard);
        let ptr = slot.load(Acquire, guard);
        unsafe { ptr.as_ref() }.map(Deref::deref)
    }

    fn insert(&self, key: u32, value: V, guard: &Guard) -> Result<(), V> {
        let slot = self.array.get(key as usize, guard);
        let node = Owned::new(Node::new(value));
        match slot.compare_exchange(Shared::null(), node, AcqRel, Acquire, guard) {
            Ok(n) => {
                // Can't change `n` to `Owned` as it is in shared memory.
                //
                // SAFETY: `n` is created in this function, hence this is the unique push of `n`.
                // Also, `n` is not used again.
                unsafe { self.storage.push_node(n, guard) };
                Ok(())
            }
            Err(e) => Err(e.new.into_box().into_inner()),
        }
    }

    fn delete<'g>(&self, key: &u32, guard: &'g Guard) -> Result<&'g V, ()> {
        let slot = self.array.get(*key as usize, guard);
        let curr = slot.load(Relaxed, guard);
        // no entry
        if curr.is_null() {
            return Err(());
        }
        match slot.compare_exchange(curr, Shared::null(), AcqRel, Acquire, guard) {
            Ok(_) => Ok(unsafe { curr.deref() }.deref()),
            Err(_) => Err(()), // already removed
        }
    }
}

mod stack {
    use core::cell::UnsafeCell;
    use core::ops::Deref;
    use core::sync::atomic::Ordering::*;
    use core::{mem, ptr};

    use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

    #[derive(Debug)]
    pub(super) struct Stack<T> {
        head: Atomic<Node<T>>,
    }

    impl<T> Stack<T> {
        pub(super) fn new() -> Self {
            Self {
                head: Atomic::null(),
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct Node<T> {
        data: T,
        next: UnsafeCell<*const Node<T>>,
    }

    impl<T> Node<T> {
        pub(super) fn new(data: T) -> Self {
            Self {
                data,
                next: UnsafeCell::new(ptr::null()),
            }
        }

        pub(super) fn into_inner(self) -> T {
            self.data
        }
    }

    impl<T> Deref for Node<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    unsafe impl<T: Send> Send for Node<T> {}
    unsafe impl<T: Sync> Sync for Node<T> {}

    impl<T> Stack<T> {
        /// This stack is used as a temporary trash can for nodes. As such, unlike the Trieber's
        /// stack in the lecture, we cannot require the full ownership of pushed nodes. So we mark
        /// it as `unsafe` to prevent the same node being pushed multiple times.
        ///
        /// # Safety
        ///
        /// - A single `n` should only be pushed into the stack once.
        /// - After the push, `n` should not be used again.
        pub(super) unsafe fn push_node<'g>(&self, n: Shared<'g, Node<T>>, guard: &'g Guard) {
            let mut head = self.head.load(Relaxed, guard);
            loop {
                unsafe { *n.deref().next.get() = head.as_raw() };

                match self.head.compare_exchange(head, n, Relaxed, Relaxed, guard) {
                    Ok(_) => break,
                    Err(e) => head = e.current,
                }
            }
        }
    }

    impl<T> Drop for Stack<T> {
        fn drop(&mut self) {
            let mut o_curr = mem::take(&mut self.head);

            while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
                o_curr = curr.next.into_inner().into();
            }
        }
    }
}

#[test]
fn smoke() {
    let list = ArrayMap::default();

    let guard = pin();

    assert_eq!(list.insert(37, 37, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), None);
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.insert(42, 42, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.delete(&37, &guard), Ok(&37));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);

    assert_eq!(list.delete(&37, &guard), Err(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);
}

#[test]
fn stress_sequential() {
    const STEPS: usize = 4096;
    map::stress_sequential::<_, _, ArrayMap<usize>>(STEPS);
}

#[test]
fn lookup_concurrent() {
    const THREADS: usize = 4;
    const STEPS: usize = 4096;
    map::lookup_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn insert_concurrent() {
    const THREADS: usize = 8;
    const STEPS: usize = 4096 * 4;
    map::insert_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 128 } else { 512 };
    map::stress_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 16 } else { 64 };
    map::log_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}
