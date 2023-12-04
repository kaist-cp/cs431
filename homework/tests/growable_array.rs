#![feature(cfg_sanitize)]

use core::mem::{self, ManuallyDrop};
use core::ptr;
use core::sync::atomic::Ordering;
use crossbeam_epoch::{pin, Atomic, Guard, Owned, Shared};

use cs431_homework::test::adt::map;
use cs431_homework::{GrowableArray, NonblockingConcurrentMap, NonblockingMap};

#[derive(Debug, Default)]
struct ArrayMap<V> {
    array: GrowableArray<Node<V>>,
    /// dump everything into a stack and drop them later
    storage: Stack<V>,
}

/// Simple map implementation using the array index as the key.
/// Uses u32 key instead of u64 to limit memory usage and runtime
impl<V> NonblockingMap<u32, V> for ArrayMap<V> {
    fn lookup<'g>(&self, key: &u32, guard: &'g Guard) -> Option<&'g V> {
        let slot = self.array.get(*key as usize, guard);
        let ptr = slot.load(Ordering::Acquire, guard);
        unsafe { ptr.as_ref() }.map(|n| &*n.data)
    }

    fn insert(&self, key: &u32, value: V, guard: &Guard) -> Result<(), V> {
        let slot = self.array.get(*key as usize, guard);
        let node = Owned::new(Node {
            data: ManuallyDrop::new(value),
            next: ptr::null(),
        });
        match slot.compare_exchange(
            Shared::null(),
            node,
            Ordering::AcqRel,
            Ordering::Acquire,
            guard,
        ) {
            Ok(n) => {
                self.storage.push_node(unsafe { n.into_owned() });
                Ok(())
            }
            Err(e) => Err(ManuallyDrop::into_inner(e.new.into_box().data)),
        }
    }

    fn delete<'g>(&self, key: &u32, guard: &'g Guard) -> Result<&'g V, ()> {
        let slot = self.array.get(*key as usize, guard);
        let curr = slot.load(Ordering::Relaxed, guard);
        // no entry
        if curr.is_null() {
            return Err(());
        }
        match slot.compare_exchange(
            curr,
            Shared::null(),
            Ordering::AcqRel,
            Ordering::Acquire,
            guard,
        ) {
            Ok(_) => Ok(unsafe { &*curr.deref().data }),
            Err(_) => Err(()), // already removed
        }
    }
}

#[derive(Debug)]
struct Stack<T> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    data: ManuallyDrop<T>,
    next: *const Node<T>,
}

unsafe impl<T: Send> Send for Node<T> {}
unsafe impl<T: Sync> Sync for Node<T> {}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            head: Atomic::null(),
        }
    }
}

impl<T> Stack<T> {
    fn push_node(&self, mut n: Owned<Node<T>>) {
        let guard = pin();

        loop {
            let head = self.head.load(Ordering::Relaxed, &guard);
            n.next = head.as_raw();

            match self
                .head
                .compare_exchange(head, n, Ordering::Release, Ordering::Relaxed, &guard)
            {
                Ok(_) => break,
                Err(e) => n = e.new,
            }
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut curr = mem::take(&mut self.head);

        while let Some(curr_ref) = unsafe { curr.try_into_owned() } {
            let curr_ref = curr_ref.into_box();
            drop(ManuallyDrop::into_inner(curr_ref.data));
            curr = curr_ref.next.into();
        }
    }
}

#[test]
fn smoke() {
    let list = ArrayMap::<usize>::default();

    let guard = pin();

    assert_eq!(list.insert(&37, 37, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), None);
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.insert(&42, 42, &guard), Ok(()));
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
    map::stress_concurrent_sequential::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(
        STEPS,
    );
}

#[test]
fn lookup_concurrent() {
    const THREADS: usize = 4;
    const STEPS: usize = 4096;
    map::lookup_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}

#[test]
fn insert_concurrent() {
    const THREADS: usize = 8;
    const STEPS: usize = 4096 * 4;
    map::insert_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 128 } else { 512 };
    map::stress_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 16 } else { 64 };
    map::log_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}
