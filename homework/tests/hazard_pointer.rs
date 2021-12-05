use core::mem::ManuallyDrop;
use core::ptr;
use std::thread::sleep;
use std::time::Duration;

#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{AtomicPtr, Ordering::*};
#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicPtr, Ordering::*};

use crossbeam_utils::thread::scope;
use cs431_homework::hazard_pointer::{collect, retire, Shield};

#[test]
fn counter() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = AtomicPtr::new(Box::leak(Box::new(0usize)));
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Box::new(0);
                    let shield = Shield::default();
                    loop {
                        let cur_ptr = shield.protect(&count);
                        let value = unsafe { *cur_ptr };
                        *new = value + 1;
                        let new_ptr = Box::leak(new);
                        if count
                            .compare_exchange(cur_ptr as *mut _, new_ptr, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_ptr as *mut usize);
                            break;
                        } else {
                            new = unsafe { Box::from_raw(new_ptr) };
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur }, THREADS * ITER);
    retire(cur);
}

// like `counter`, but trigger interesting interleaving using `sleep` and always call `collect`.
#[test]
fn counter_sleep() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = AtomicPtr::new(Box::leak(Box::new(0usize)));
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Box::new(0);
                    let shield = Shield::default();
                    loop {
                        let mut cur_ptr = count.load(Relaxed) as *const _;
                        while !shield.try_protect(&mut cur_ptr, &count) {
                            sleep(Duration::from_micros(1));
                        }
                        let value = unsafe { *cur_ptr };
                        *new = value + 1;
                        let new_ptr = Box::leak(new);
                        if count
                            .compare_exchange(cur_ptr as *mut _, new_ptr, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_ptr as *mut usize);
                            collect();
                            break;
                        } else {
                            new = unsafe { Box::from_raw(new_ptr) };
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur }, THREADS * ITER);
    retire(cur);
}

#[test]
fn stack() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 16;

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
    const ITER: usize = 1024 * 16;

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
pub struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    data: ManuallyDrop<T>,
    next: *const Node<T>,
}

unsafe impl<T: Send> Send for Node<T> {}
unsafe impl<T: Sync> Sync for Node<T> {}

impl<T> Stack<T> {
    /// Creates a new, empty stack.
    pub fn new() -> Stack<T> {
        Stack {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Pushes a value on top of the stack.
    pub fn push(&self, t: T) {
        let new = Box::leak(Box::new(Node {
            data: ManuallyDrop::new(t),
            next: ptr::null(),
        }));

        loop {
            let head = self.head.load(Relaxed);
            new.next = head;

            if self
                .head
                .compare_exchange(head, new, Release, Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Attempts to pop the top element from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&self) -> Option<T> {
        let shield = Shield::default();
        loop {
            let head_ptr = shield.protect(&self.head);
            let head_ref = unsafe { head_ptr.as_ref()? };

            if self
                .head
                .compare_exchange(
                    head_ptr as *mut _,
                    head_ref.next as *mut _,
                    Relaxed,
                    Relaxed,
                )
                .is_ok()
            {
                let head_ptr = head_ptr as *mut Node<T>;
                unsafe {
                    let data = ManuallyDrop::take(&mut (*head_ptr).data);
                    retire(head_ptr);
                    return Some(data);
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
    use super::mock::sync::atomic::{AtomicPtr, AtomicUsize, Ordering::*};
    use super::mock::sync::Arc;
    use super::mock::thread;
    use core::ptr;
    use cs431_homework::hazard_pointer::*;

    #[test]
    fn try_protect_collect_sync() {
        model(|| {
            let atomic = Arc::new(AtomicPtr::new(Box::leak(Box::new(123usize))));

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let mut local = atomic.load(Relaxed) as *const usize;
                    if local.is_null() {
                        return;
                    }
                    let shield = Shield::default();
                    if shield.try_protect(&mut local, &atomic) {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *local }, 123);
                    }
                })
            };

            // unlink, retire, and collect
            let local = atomic.load(Relaxed);
            atomic.store(ptr::null_mut(), Relaxed);
            retire(local);
            collect();
            th.join().unwrap();
        })
    }

    #[test]
    fn protect_collect_sync() {
        model(|| {
            let atomic = Arc::new(AtomicPtr::new(ptr::null_mut::<usize>()));

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let shield = Shield::default();
                    let local = shield.protect(&atomic);
                    if !local.is_null() {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *local }, 123);
                    }
                })
            };

            // link
            let local = Box::into_raw(Box::new(123));
            atomic.store(local, Release);

            // unlink, retire, and collect
            atomic.store(ptr::null_mut(), Relaxed);
            retire(local);
            collect();

            th.join().unwrap();
        })
    }

    // Above tests can't detect the absence of release-acquire between `Shield::drop` and `collect`
    // for an unknown reasone. So explicitly check release-acquire between `Shield::drop` and
    // `all_hazards`.
    #[test]
    fn shield_drop_all_hazards_sync() {
        model(|| {
            let obj = Box::into_raw(Box::new(AtomicUsize::new(0)));
            let atomic = Arc::new(AtomicPtr::new(obj));
            let obj = obj as usize;
            let shield = Shield::default();
            let local = shield.protect(&atomic);

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let local = atomic.load(Relaxed);
                    if !HAZARDS.all_hazards().contains(&obj) {
                        unsafe { assert_eq!((*local).load(Relaxed), 123) };
                    }
                })
            };

            unsafe { (*local).store(123, Relaxed) };
            drop(shield);

            th.join().unwrap();
            unsafe { drop(Box::from_raw(local as *mut AtomicUsize)) };
        })
    }
}
