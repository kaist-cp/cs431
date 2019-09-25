use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::thread::{self, Thread};

use crossbeam_utils::CachePadded;

use crate::lock::*;

struct Node {
    thread: Thread,
    locked: AtomicBool,
    next: AtomicPtr<CachePadded<Node>>,
    _marker: PhantomData<*const ()>,
}

#[derive(Clone)]
pub struct Token(*mut CachePadded<Node>);

pub struct McsParkingLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    fn new() -> Self {
        Self {
            thread: thread::current(),
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
            _marker: PhantomData,
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
        let prev = self.tail.swap(node, Ordering::Relaxed);

        if prev.is_null() {
            return Token(node);
        }

        unsafe {
            (*prev).next.store(node, Ordering::Release);
        }

        while unsafe { (*node).locked.load(Ordering::Acquire) } {
            thread::park();
        }

        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        let node = token.0;

        loop {
            let next = (*node).next.load(Ordering::Acquire);
            if !next.is_null() {
                drop(Box::from_raw(node));
                let thread = (*next).thread.clone();
                (*next).locked.store(false, Ordering::Release);
                thread.unpark();
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
    use crate::mcsparkinglock::McsParkingLock;

    #[test]
    fn smoke() {
        crate::lock::tests::smoke::<McsParkingLock>();
    }
}
