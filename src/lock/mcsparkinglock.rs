use core::ptr;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{AtomicBool, AtomicPtr};
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
    fn new() -> *mut CachePadded<Self> {
        Box::into_raw(Box::new(CachePadded::new(Self {
            thread: thread::current(),
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
        })))
    }
}

impl Default for McsParkingLock {
    fn default() -> Self {
        Self {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

unsafe impl RawLock for McsParkingLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Node::new();
        let prev = self.tail.swap(node, AcqRel);

        if prev.is_null() {
            return Token(node);
        }

        // SAFETY: See safety of McsLock::lock().
        unsafe { (*prev).next.store(node, Release) };

        // SAFETY: See safety of McsLock::lock().
        while unsafe { (*node).locked.load(Acquire) } {
            thread::park();
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
                // SAFETY: See safety of McsLock::unlock().
                drop(unsafe { Box::from_raw(node) });
                return;
            }

            while {
                next = unsafe { (*node).next.load(Acquire) };
                next.is_null()
            } {}
        }

        // SAFETY: See safety of McsLock::unlock().
        drop(unsafe { Box::from_raw(node) });
        let next_ref = unsafe { &*next };
        let thread = next_ref.thread.clone();
        next_ref.locked.store(false, Release);
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
