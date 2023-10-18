use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::thread::{self, Thread};

use crossbeam_utils::CachePadded;

use crate::lock::*;

struct Node {
    thread: Thread,
    locked: AtomicBool,
    next: AtomicPtr<CachePadded<Node>>,
}

#[derive(Debug, Clone)]
pub struct Token(*mut CachePadded<Node>);

/// An MCS parking lock.
#[derive(Debug)]
pub struct McsParkingLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    fn new() -> Self {
        Self {
            thread: thread::current(),
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl Default for McsParkingLock {
    fn default() -> Self {
        Self {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl RawLock for McsParkingLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Box::into_raw(Box::new(CachePadded::new(Node::new())));
        let prev = self.tail.swap(node, Ordering::AcqRel);

        if prev.is_null() {
            return Token(node);
        }

        // SAFETY: See safety of McsLock::lock().
        unsafe { (*prev).next.store(node, Ordering::Release) };

        // SAFETY: See safety of McsLock::lock().
        while unsafe { (*node).locked.load(Ordering::Acquire) } {
            thread::park();
        }

        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        let node = token.0;
        let mut next = (*node).next.load(Ordering::Acquire);

        if next.is_null() {
            if self
                .tail
                .compare_exchange(node, ptr::null_mut(), Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                // SAFETY: See safety of McsLock::unlock().
                drop(Box::from_raw(node));
                return;
            }

            while {
                next = (*node).next.load(Ordering::Acquire);
                next.is_null()
            } {}
        }

        // SAFETY: See safety of McsLock::unlock().
        drop(Box::from_raw(node));
        let thread = (*next).thread.clone();
        (*next).locked.store(false, Ordering::Release);
        thread.unpark();
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::mcsparkinglock::McsParkingLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<McsParkingLock>();
    }
}
