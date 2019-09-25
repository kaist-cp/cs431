use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
}

#[derive(Clone)]
pub struct Token(*const CachePadded<Node>);

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
        let prev = self.tail.swap(node, Ordering::Relaxed);
        let backoff = Backoff::new();

        while unsafe { (*prev).locked.load(Ordering::Acquire) } {
            backoff.snooze();
        }

        drop(unsafe { Box::from_raw(prev) });
        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        (*token.0).locked.store(false, Ordering::Release);
    }
}
