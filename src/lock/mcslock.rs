use core::ptr;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{AtomicBool, AtomicPtr};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
    next: AtomicPtr<CachePadded<Node>>,
}

#[derive(Debug, Clone)]
pub struct Token(*mut CachePadded<Node>);

/// An MCS lock.
#[derive(Debug)]
pub struct McsLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    fn new() -> *mut CachePadded<Self> {
        Box::into_raw(Box::new(CachePadded::new(Self {
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
        })))
    }
}

impl Default for McsLock {
    fn default() -> Self {
        Self {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

unsafe impl RawLock for McsLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Node::new();
        let prev = self.tail.swap(node, AcqRel);

        if prev.is_null() {
            return Token(node);
        }

        // SAFETY: `prev` is valid, so is not the initial pointer. Hence, it is a pointer from
        // `swap()` by another thread's `lock()`, and that thread guarantees that `prev` will not be
        // freed until this store is complete.
        unsafe { (*prev).next.store(node, Release) };

        let backoff = Backoff::new();
        // SAFETY: `node` was made valid above. Since other threads will not free `node`, it still
        // points to valid memory.
        while unsafe { (*node).locked.load(Acquire) } {
            backoff.snooze();
        }

        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        let node = token.0;
        let mut next = unsafe { (*node).next.load(Acquire) };

        if next.is_null() {
            if self
                .tail
                .compare_exchange(node, ptr::null_mut(), Release, Relaxed)
                .is_ok()
            {
                // SAFETY: Since `node` was the `tail`, there is no other thread blocked by this
                // lock. Hence we have unique access to it.
                drop(unsafe { Box::from_raw(node) });
                return;
            }

            while {
                next = unsafe { (*node).next.load(Acquire) };
                next.is_null()
            } {}
        }

        // SAFETY: Since `next` is not null, the thread that made `next` has finished access to
        // `node`, hence we have unique access to it.
        drop(unsafe { Box::from_raw(node) });
        unsafe { (*next).locked.store(false, Release) };
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::McsLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<McsLock>();
    }
}
