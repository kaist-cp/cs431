use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct Token(*const CachePadded<Node>);

/// CLH lock.
#[derive(Debug)]
pub struct ClhLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    const fn new(locked: bool) -> Self {
        Self {
            locked: AtomicBool::new(locked),
        }
    }
}

impl Default for ClhLock {
    fn default() -> Self {
        let node = AtomicPtr::new(Box::into_raw(Box::new(CachePadded::new(Node::new(false)))));

        Self { tail: node }
    }
}

impl RawLock for ClhLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Box::into_raw(Box::new(CachePadded::new(Node::new(true))));
        let prev = self.tail.swap(node, Ordering::AcqRel);
        let backoff = Backoff::new();

        // SAFETY: `prev` is valid, as `self.tail` was valid at initialization and any `swap()` to
        // it by other `lock()`s. Hence, it points to valid memory as the thread that made
        // `prev` will not free it.
        while unsafe { (*prev).locked.load(Ordering::Acquire) } {
            backoff.snooze();
        }

        // SAFETY: since `prev` was obtained from a swap on tail, only this thread other
        // than its creator can access it. Since the creator will no longer access `prev` as its
        // `locked` is false, we have unique access to it.
        drop(unsafe { Box::from_raw(prev) });
        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        (*token.0).locked.store(false, Ordering::Release);
    }
}

impl Drop for ClhLock {
    fn drop(&mut self) {
        // Drop the node made by the last thread that `lock()`ed.
        let node = self.tail.load(Ordering::Relaxed);

        // SAFETY: Since this is the tail node, no other thread has access to it.
        drop(unsafe { Box::from_raw(node) });
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::ClhLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<ClhLock>();
    }
}
