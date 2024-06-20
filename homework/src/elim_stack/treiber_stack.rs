use core::mem::{self, ManuallyDrop};
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

use super::base::Stack;

#[derive(Debug)]
pub struct Node<T> {
    data: ManuallyDrop<T>,
    next: *const Node<T>,
}

// Any particular `T` should never be accessed concurrently, so no need for `Sync`.
unsafe impl<T: Send> Send for Node<T> {}
unsafe impl<T: Send> Sync for Node<T> {}

/// Treiber's lock-free stack.
///
/// Usable with any number of producers and consumers.
#[derive(Debug)]
pub struct TreiberStack<T> {
    head: Atomic<Node<T>>,
}

impl<T> From<T> for Node<T> {
    fn from(t: T) -> Self {
        Self {
            data: ManuallyDrop::new(t),
            next: ptr::null(),
        }
    }
}

impl<T> Deref for Node<T> {
    type Target = ManuallyDrop<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Default for TreiberStack<T> {
    fn default() -> Self {
        TreiberStack {
            head: Atomic::null(),
        }
    }
}

impl<T> Stack<T> for TreiberStack<T> {
    type PushReq = Node<T>;

    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>> {
        let mut req = req;
        let head = self.head.load(Ordering::Relaxed, guard);
        req.next = head.as_raw();

        match self
            .head
            .compare_exchange(head, req, Ordering::Release, Ordering::Relaxed, guard)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.new),
        }
    }

    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        let head = self.head.load(Ordering::Acquire, guard);
        let Some(head_ref) = (unsafe { head.as_ref() }) else {
            return Ok(None);
        };
        let next = Shared::from(head_ref.next);

        let _ = self
            .head
            .compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed, guard)
            .map_err(|_| ())?;

        let data = ManuallyDrop::into_inner(unsafe { ptr::read(&head_ref.data) });
        unsafe { guard.defer_destroy(head) };
        Ok(Some(data))
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.head.load(Ordering::Acquire, guard).is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        let mut o_curr = mem::take(&mut self.head);
        while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
            drop(ManuallyDrop::into_inner(curr.data));
            o_curr = curr.next.into();
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread::scope;

    use super::*;

    #[test]
    fn push() {
        let stack = TreiberStack::default();

        scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..10 {
                let handle = scope.spawn(|| {
                    for i in 0..10_000 {
                        stack.push(i);
                        assert!(stack.pop().is_some());
                    }
                });
                handles.push(handle);
            }
        });

        assert!(stack.pop().is_none());
    }
}
