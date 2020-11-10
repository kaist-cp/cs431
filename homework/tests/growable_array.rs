use core::mem::{replace, ManuallyDrop};
use core::sync::atomic::Ordering;
use crossbeam_epoch::{pin, unprotected, Atomic, Guard, Owned, Shared};
use cs492_concur_homework::{GrowableArray, NonblockingConcurrentMap, NonblockingMap};

mod map;

#[derive(Debug, Default)]
struct ArrayMap<V> {
    array: GrowableArray<Node<V>>,
    /// dump everything into a stack and drop them later
    storage: Stack<V>,
}

/// Simple map implementation using array index as key.
/// Uses u32 key instead of u60 to limit memory usage and runtime
impl<V> NonblockingMap<u32, V> for ArrayMap<V> {
    fn lookup<'g>(&self, key: &u32, guard: &'g Guard) -> Option<&'g V> {
        let slot = self.array.get(*key as usize, guard);
        let ptr = slot.load(Ordering::Acquire, guard);
        unsafe { ptr.as_ref().map(|n| &*n.data) }
    }

    fn insert(&self, key: &u32, value: V, guard: &Guard) -> Result<(), V> {
        let slot = self.array.get(*key as usize, guard);
        let node = Owned::new(Node {
            data: ManuallyDrop::new(value),
            next: Atomic::null(),
        });
        match slot.compare_and_set(Shared::null(), node, Ordering::AcqRel, guard) {
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
        match slot.compare_and_set(curr, Shared::null(), Ordering::AcqRel, guard) {
            Ok(_) => Ok(unsafe { &*curr.as_ref().unwrap().data }),
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
    next: Atomic<Node<T>>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            head: Atomic::null(),
        }
    }
}

impl<T> Stack<T> {
    fn push_node(&self, mut n: Owned<Node<T>>) {
        let guard = crossbeam_epoch::pin();

        loop {
            let head = self.head.load(Ordering::Relaxed, &guard);
            n.next.store(head, Ordering::Relaxed);

            match self
                .head
                .compare_and_set(head, n, Ordering::Release, &guard)
            {
                Ok(_) => break,
                Err(e) => n = e.new,
            }
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        unsafe {
            let guard = unprotected();
            let mut cur = self.head.load(Ordering::Relaxed, guard);
            while let Some(n) = cur.as_ref() {
                let next = n.next.load(Ordering::Relaxed, guard);
                drop(replace(&mut cur, next).into_owned());
            }
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
fn stress_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096;
    map::stress_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 12;
    map::log_concurrent::<u32, NonblockingConcurrentMap<_, _, ArrayMap<usize>>>(THREADS, STEPS);
}
