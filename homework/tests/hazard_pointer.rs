use core::mem::ManuallyDrop;
use core::ptr;
use core::sync::atomic::Ordering::*;

use crossbeam_utils::thread::scope;
use cs492_concur_homework::hazard_pointer::{get_protected, retire, Atomic, Owned};

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

// NOTE: more tests will be added soon™
