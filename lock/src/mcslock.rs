use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
    next: AtomicPtr<CachePadded<Node>>,
}

#[derive(Clone)]
pub struct Token(*mut CachePadded<Node>);

pub struct McsLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl Default for McsLock {
    fn default() -> Self {
        Self {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl RawLock for McsLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Box::into_raw(Box::new(CachePadded::new(Node::new())));
        let prev = self.tail.swap(node, Ordering::Relaxed);

        if prev.is_null() {
            return Token(node);
        }

        unsafe {
            (*prev).next.store(node, Ordering::Release);
        }

        let backoff = Backoff::new();
        while unsafe { (*node).locked.load(Ordering::Acquire) } {
            backoff.snooze();
        }

        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        let node = token.0;

        loop {
            let next = (*node).next.load(Ordering::Acquire);
            if !next.is_null() {
                drop(Box::from_raw(node));
                (*next).locked.store(false, Ordering::Release);
                return;
            }

            if self
                .tail
                .compare_exchange(node, ptr::null_mut(), Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                drop(Box::from_raw(node));
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mcslock::McsLock;

    #[test]
    fn smoke() {
        crate::lock::tests::smoke::<McsLock>();
    }
}
