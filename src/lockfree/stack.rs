use core::mem::{self, ManuallyDrop};
use core::ptr;
use core::sync::atomic::Ordering;

use crossbeam_epoch::{Atomic, Owned, Shared};

/// Treiber's lock-free stack.
///
/// Usable with any number of producers and consumers.
#[derive(Debug)]
pub struct Stack<T> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    data: ManuallyDrop<T>,
    next: *const Node<T>,
}

// Any particular `T` should never be accessed concurrently, so no need for `Sync`.
unsafe impl<T: Send> Send for Stack<T> {}
unsafe impl<T: Send> Sync for Stack<T> {}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            head: Atomic::null(),
        }
    }
}

impl<T> Stack<T> {
    /// Creates a new, empty stack.
    pub fn new() -> Stack<T> {
        Self::default()
    }

    /// Pushes a value on top of the stack.
    pub fn push(&self, t: T) {
        let mut n = Owned::new(Node {
            data: ManuallyDrop::new(t),
            next: ptr::null(),
        });

        let guard = crossbeam_epoch::pin();

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

    /// Attempts to pop the top element from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&self) -> Option<T> {
        let guard = crossbeam_epoch::pin();
        loop {
            let head = self.head.load(Ordering::Acquire, &guard);
            let h = unsafe { head.as_ref() }?;
            let next = Shared::from(h.next);

            if self
                .head
                .compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed, &guard)
                .is_ok()
            {
                // Since the above `compare_exchange()` succeeded, `head` is detached from `self` so
                // is unreachable from other threads.

                // SAFETY: We are returning ownership of `data` in `head` by making a copy of it via
                // `ptr::read()`. This is safe as no other thread has access to `data` after `head`
                // is unreachable, so the ownership of `data` in `head` will never be used again.
                let result = ManuallyDrop::into_inner(unsafe { ptr::read(&h.data) });

                // SAFETY: `head` is unreachable, and we no longer access `head`.
                unsafe { guard.defer_destroy(head) };

                return Some(result);
            }
        }
    }

    /// Returns `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        let guard = crossbeam_epoch::pin();
        self.head.load(Ordering::Acquire, &guard).is_null()
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut o_curr = mem::take(&mut self.head);

        // SAFETY: All non-null nodes made were valid, and we have unique ownership via `&mut self`.
        while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
            drop(ManuallyDrop::into_inner(curr.data));
            o_curr = curr.next.into();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread::scope;

    #[test]
    fn push() {
        let stack = Stack::new();

        scope(|scope| {
            for _ in 0..10 {
                scope.spawn(|| {
                    for i in 0..10_000 {
                        stack.push(i);
                        assert!(stack.pop().is_some());
                    }
                });
            }
        });

        assert!(stack.pop().is_none());
    }
}
