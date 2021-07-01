use core::mem::ManuallyDrop;
use core::ptr;
use core::sync::atomic::Ordering::*;
use std::thread::sleep;
use std::time::Duration;

use crossbeam_utils::thread::scope;
use cs431_homework::hazard_pointer::{collect, get_protected, protect, retire, Atomic, Owned};

#[test]
fn counter() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = Atomic::new(0usize);
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Owned::new(0);
                    loop {
                        let cur_shield = get_protected(&count).unwrap();
                        let value = unsafe { *cur_shield.deref() };
                        *new = value + 1;
                        let new_shared = new.into_shared();
                        if count
                            .compare_and_set(cur_shield.shared(), new_shared, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_shield.shared());
                            break;
                        } else {
                            new = unsafe { new_shared.into_owned() };
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur.deref() }, THREADS * ITER);
    retire(cur);
}

// like `counter`, but trigger interesting interleaving using `sleep` and always call `collect`.
#[test]
fn counter_sleep() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = Atomic::new(0usize);
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Owned::new(0);
                    loop {
                        let mut shared = count.load(Relaxed);
                        let cur_shield = loop {
                            sleep(Duration::from_micros(1));
                            let shield = protect(shared).unwrap();
                            let shared2 = count.load(Relaxed);
                            if shield.validate(shared2) {
                                break shield;
                            }
                            shared = shared2;
                        };
                        let value = unsafe { *cur_shield.deref() };
                        *new = value + 1;
                        let new_shared = new.into_shared();
                        if count
                            .compare_and_set(cur_shield.shared(), new_shared, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_shield.shared());
                            collect();
                            break;
                        } else {
                            new = unsafe { new_shared.into_owned() };
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur.deref() }, THREADS * ITER);
    retire(cur);
}

#[test]
fn counter_tag() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = Atomic::new(0usize);
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Owned::new(0).with_tag(1);
                    loop {
                        let cur_shield = get_protected(&count).unwrap();
                        let value = unsafe { *cur_shield.deref() };
                        *new = value + 1;
                        let new_shared = new.into_shared();
                        if count
                            .compare_and_set(cur_shield.shared(), new_shared, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_shield.shared());
                            break;
                        } else {
                            new = unsafe { new_shared.into_owned() };
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur.deref() }, THREADS * ITER);
    retire(cur);
}

#[test]
fn stack() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 8;

    let stack = Stack::new();
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for i in 0..ITER {
                    stack.push(i);
                    stack.pop();
                }
            });
        }
    })
    .unwrap();
    assert!(stack.pop().is_none());
}

#[test]
fn two_stacks() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 8;

    let stack1 = Stack::new();
    let stack2 = Stack::new();
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for i in 0..ITER {
                    stack1.push(i);
                    stack1.pop();
                    stack2.push(i);
                    stack2.pop();
                }
            });
        }
    })
    .unwrap();
    assert!(stack1.pop().is_none());
}

/// Treiber's lock-free stack.
///
/// Usable with any number of producers and consumers.
#[derive(Debug)]
pub struct Stack<T: 'static> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    data: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

impl<T: 'static> Stack<T> {
    /// Creates a new, empty stack.
    pub fn new() -> Stack<T> {
        Stack {
            head: Atomic::null(),
        }
    }

    /// Pushes a value on top of the stack.
    pub fn push(&self, t: T) {
        let n = Owned::new(Node {
            data: ManuallyDrop::new(t),
            next: Atomic::null(),
        })
        .into_shared();

        loop {
            let head = self.head.load(Relaxed);
            unsafe { n.deref() }.next.store(head, Relaxed);

            if self.head.compare_and_set(head, n, Release, Relaxed).is_ok() {
                break;
            }
        }
    }

    /// Attempts to pop the top element from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&self) -> Option<T> {
        loop {
            let head_shield = get_protected(&self.head).unwrap();

            let next = unsafe { head_shield.as_ref()? }.next.load(Relaxed);

            if self
                .head
                .compare_and_set(head_shield.shared(), next, Relaxed, Relaxed)
                .is_ok()
            {
                unsafe {
                    retire(head_shield.shared());
                    return Some(ManuallyDrop::into_inner(ptr::read(
                        &(*head_shield.deref()).data,
                    )));
                }
            }
        }
    }

    /// Returns `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.head.load(Acquire).is_null()
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

mod mock;

mod sync {
    use super::mock::model;
    use super::mock::sync::atomic::{AtomicUsize, Ordering::*};
    use super::mock::sync::Arc;
    use super::mock::thread;
    use cs431_homework::hazard_pointer::*;

    #[test]
    fn protect_collect_sync() {
        model(|| {
            let atomic = Arc::new(Atomic::new(123));

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let shared = atomic.load(Relaxed);
                    if shared.is_null() {
                        return;
                    }
                    let shield = protect(shared).unwrap();
                    if shield.validate(atomic.load(Relaxed)) {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *shield.deref() }, 123);
                    }
                })
            };

            // unlink, retire, and collect
            let shared = atomic.load(Relaxed);
            atomic.store(Shared::null(), Relaxed);
            retire(shared);
            collect();
            th.join().unwrap();
        })
    }

    #[test]
    fn get_protected_collect_sync() {
        model(|| {
            let atomic = Arc::new(Atomic::null());

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let shield = get_protected(&atomic).unwrap();
                    if !shield.is_null() {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *shield.deref() }, 123);
                    }
                })
            };

            // link
            let shared = Owned::new(123).into_shared();
            atomic.store(shared, Release);

            // unlink, retire, and collect
            atomic.store(Shared::null(), Relaxed);
            retire(shared);
            collect();

            th.join().unwrap();
        })
    }

    // Above tests can't detect the absence of release-acquire between `Shield::drop` and `collect`
    // probably because loom doesn't support promise. So explicitly check release-acquire between
    // `Shield::drop` and `all_hazards`.
    #[test]
    fn shield_drop_all_hazards_sync() {
        model(|| {
            let atomic = Arc::new(Atomic::new(AtomicUsize::new(0)));
            let shield = protect(atomic.load(Relaxed)).unwrap();

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let shared = atomic.load(Relaxed);
                    // shield drop happens before all_hazards
                    if HAZARDS.all_hazards().is_empty() {
                        unsafe { assert_eq!(shared.deref().load(Relaxed), 123) };
                    }
                })
            };

            unsafe { shield.deref().store(123, Relaxed) };
            let shared = shield.shared();
            drop(shield);

            th.join().unwrap();
            unsafe { drop(shared.into_owned()) };
        })
    }
}
