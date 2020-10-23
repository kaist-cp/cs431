//! Michael-Scott lock-free queue.
//!
//! Usable with any number of producers and consumers.
//!
//! Michael and Scott.  Simple, Fast, and Practical Non-Blocking and Blocking Concurrent Queue
//! Algorithms.  PODC 1996.  http://dl.acm.org/citation.cfm?id=248106

use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::Ordering;

use crossbeam_epoch::{unprotected, Atomic, Guard, Owned, Shared};
use crossbeam_utils::CachePadded;

/// Michael-Scott queue.
// The representation here is a singly-linked list, with a sentinel node at the front. In general
// the `tail` pointer may lag behind the actual tail. Non-sentinel nodes are either all `Data` or
// all `Blocked` (requests for data from blocked threads).
#[derive(Debug)]
pub struct Queue<T> {
    head: CachePadded<Atomic<Node<T>>>,
    tail: CachePadded<Atomic<Node<T>>>,
}

#[derive(Debug)]
struct Node<T> {
    /// The slot in which a value of type `T` can be stored.
    ///
    /// The type of `data` is `ManuallyDrop<T>` because a `Node<T>` doesn't always contain a `T`.
    /// For example, the sentinel node in a queue never contains a value: its slot is always empty.
    /// Other nodes start their life with a push operation and contain a value until it gets popped
    /// out. After that such empty nodes get added to the collector for destruction.
    data: MaybeUninit<T>,

    next: Atomic<Node<T>>,
}

// Any particular `T` should never be accessed concurrently, so no need for `Sync`.
unsafe impl<T: Send> Sync for Queue<T> {}
unsafe impl<T: Send> Send for Queue<T> {}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        let q = Self {
            head: CachePadded::new(Atomic::null()),
            tail: CachePadded::new(Atomic::null()),
        };
        // TODO(taiki-e): when the minimum supported Rust version is bumped to 1.36+,
        // replace this with `mem::MaybeUninit`.
        #[allow(deprecated)]
        let sentinel = Owned::new(Node {
            data: MaybeUninit::uninit(),
            next: Atomic::null(),
        });
        unsafe {
            let guard = &unprotected();
            let sentinel = sentinel.into_shared(guard);
            q.head.store(sentinel, Ordering::Relaxed);
            q.tail.store(sentinel, Ordering::Relaxed);
            q
        }
    }
}

impl<T> Queue<T> {
    /// Create a new, empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `t` to the back of the queue, possibly waking up threads blocked on `pop`.
    pub fn push(&self, t: T, guard: &Guard) {
        let new = Owned::new(Node {
            data: MaybeUninit::new(t),
            next: Atomic::null(),
        });
        let new = Owned::into_shared(new, guard);

        loop {
            // We push onto the tail, so we'll start optimistically by looking there first.
            let tail = self.tail.load(Ordering::Acquire, guard);

            // Attempt to push onto the `tail` snapshot; fails if `tail.next` has changed.
            let tail_ref = unsafe { tail.deref() };
            let next = tail_ref.next.load(Ordering::Acquire, guard);

            // If `tail` is not the actual tail, try to "help" by moving the tail pointer forward.
            if !next.is_null() {
                let _ = self
                    .tail
                    .compare_and_set(tail, next, Ordering::Release, guard);
                continue;
            }

            // looks like the actual tail; attempt to link at `tail.next`.
            if tail_ref
                .next
                .compare_and_set(Shared::null(), new, Ordering::Release, guard)
                .is_ok()
            {
                // try to move the tail pointer forward.
                let _ = self
                    .tail
                    .compare_and_set(tail, new, Ordering::Release, guard);
                break;
            }
        }
    }

    /// Attempts to dequeue from the front.
    ///
    /// Returns `None` if the queue is observed to be empty.
    pub fn try_pop(&self, guard: &Guard) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire, guard);
            let h = unsafe { head.deref() };
            let next = h.next.load(Ordering::Acquire, guard);
            let next_ref = some_or!(unsafe { next.as_ref() }, return None);

            // Moves `tail` if it's stale. Relaxed load is enough because if tail == head, then the
            // messages for that node are already acquired.
            let tail = self.tail.load(Ordering::Relaxed, guard);
            if tail == head {
                let _ = self
                    .tail
                    .compare_and_set(tail, next, Ordering::Release, guard);
            }

            if self
                .head
                .compare_and_set(head, next, Ordering::Release, guard)
                .is_ok()
            {
                unsafe {
                    guard.defer_destroy(head);
                    return Some(ptr::read(&next_ref.data).assume_init());
                }
            }
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe {
            let guard = unprotected();

            while self.try_pop(guard).is_some() {}

            // Destroy the remaining sentinel node.
            let sentinel = self.head.load(Ordering::Relaxed, guard);
            drop(sentinel.into_owned());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crossbeam_epoch::pin;
    use crossbeam_utils::thread;

    struct Queue<T> {
        queue: super::Queue<T>,
    }

    impl<T> Queue<T> {
        pub fn new() -> Queue<T> {
            Queue {
                queue: super::Queue::new(),
            }
        }

        pub fn push(&self, t: T) {
            let guard = &pin();
            self.queue.push(t, guard);
        }

        pub fn is_empty(&self) -> bool {
            let guard = &pin();
            let head = self.queue.head.load(Ordering::Acquire, guard);
            let h = unsafe { head.deref() };
            h.next.load(Ordering::Acquire, guard).is_null()
        }

        pub fn try_pop(&self) -> Option<T> {
            let guard = &pin();
            self.queue.try_pop(guard)
        }

        pub fn pop(&self) -> T {
            loop {
                match self.try_pop() {
                    None => continue,
                    Some(t) => return t,
                }
            }
        }
    }

    const CONC_COUNT: i64 = 1000000;

    #[test]
    fn push_try_pop_1() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        assert!(!q.is_empty());
        assert_eq!(q.try_pop(), Some(37));
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_2() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        q.push(48);
        assert_eq!(q.try_pop(), Some(37));
        assert!(!q.is_empty());
        assert_eq!(q.try_pop(), Some(48));
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_many_seq() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        for i in 0..200 {
            q.push(i)
        }
        assert!(!q.is_empty());
        for i in 0..200 {
            assert_eq!(q.try_pop(), Some(i));
        }
        assert!(q.is_empty());
    }

    #[test]
    fn push_pop_1() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        assert!(!q.is_empty());
        assert_eq!(q.pop(), 37);
        assert!(q.is_empty());
    }

    #[test]
    fn push_pop_2() {
        let q: Queue<i64> = Queue::new();
        q.push(37);
        q.push(48);
        assert_eq!(q.pop(), 37);
        assert_eq!(q.pop(), 48);
    }

    #[test]
    fn push_pop_many_seq() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        for i in 0..200 {
            q.push(i)
        }
        assert!(!q.is_empty());
        for i in 0..200 {
            assert_eq!(q.pop(), i);
        }
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_many_spsc() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());

        thread::scope(|scope| {
            scope.spawn(|_| {
                let mut next = 0;

                while next < CONC_COUNT {
                    if let Some(elem) = q.try_pop() {
                        assert_eq!(elem, next);
                        next += 1;
                    }
                }
            });

            for i in 0..CONC_COUNT {
                q.push(i)
            }
        })
        .unwrap();
    }

    #[test]
    fn push_try_pop_many_spmc() {
        fn recv(_t: i32, q: &Queue<i64>) {
            let mut cur = -1;
            for _i in 0..CONC_COUNT {
                if let Some(elem) = q.try_pop() {
                    assert!(elem > cur);
                    cur = elem;

                    if cur == CONC_COUNT - 1 {
                        break;
                    }
                }
            }
        }

        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        thread::scope(|scope| {
            for i in 0..3 {
                let q = &q;
                scope.spawn(move |_| recv(i, q));
            }

            scope.spawn(|_| {
                for i in 0..CONC_COUNT {
                    q.push(i);
                }
            });
        })
        .unwrap();
    }

    #[test]
    fn push_try_pop_many_mpmc() {
        enum LR {
            Left(i64),
            Right(i64),
        }

        let q: Queue<LR> = Queue::new();
        assert!(q.is_empty());

        thread::scope(|scope| {
            for _t in 0..2 {
                scope.spawn(|_| {
                    for i in CONC_COUNT - 1..CONC_COUNT {
                        q.push(LR::Left(i))
                    }
                });
                scope.spawn(|_| {
                    for i in CONC_COUNT - 1..CONC_COUNT {
                        q.push(LR::Right(i))
                    }
                });
                scope.spawn(|_| {
                    let mut vl = vec![];
                    let mut vr = vec![];
                    for _i in 0..CONC_COUNT {
                        match q.try_pop() {
                            Some(LR::Left(x)) => vl.push(x),
                            Some(LR::Right(x)) => vr.push(x),
                            _ => {}
                        }
                    }

                    let mut vl2 = vl.clone();
                    let mut vr2 = vr.clone();
                    vl2.sort();
                    vr2.sort();

                    assert_eq!(vl, vl2);
                    assert_eq!(vr, vr2);
                });
            }
        })
        .unwrap();
    }

    #[test]
    fn push_pop_many_spsc() {
        let q: Queue<i64> = Queue::new();

        thread::scope(|scope| {
            scope.spawn(|_| {
                let mut next = 0;
                while next < CONC_COUNT {
                    assert_eq!(q.pop(), next);
                    next += 1;
                }
            });

            for i in 0..CONC_COUNT {
                q.push(i)
            }
        })
        .unwrap();
        assert!(q.is_empty());
    }

    #[test]
    fn is_empty_dont_pop() {
        let q: Queue<i64> = Queue::new();
        q.push(20);
        q.push(20);
        assert!(!q.is_empty());
        assert!(!q.is_empty());
        assert!(q.try_pop().is_some());
    }
}
