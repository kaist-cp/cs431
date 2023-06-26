use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;

use crossbeam_epoch::{unprotected, Atomic, Guard, Owned};

use super::base::Stack;

#[derive(Debug)]
pub struct Node<T> {
    data: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

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
            next: Atomic::null(),
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
        let head = self.head.load(Ordering::Relaxed, guard);
        req.next.store(head, Ordering::Relaxed);
        self.head
            .compare_exchange(head, req, Ordering::Release, Ordering::Relaxed, guard)
            .map(|_| ())
            .map_err(|e| e.new)
    }

    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        let head = self.head.load(Ordering::Acquire, guard);
        let Some(head_ref) = (unsafe { head.as_ref() }) else {
                   return Ok(None);
        };
        let next = head_ref.next.load(Ordering::Relaxed, guard);

        let _ = self
            .head
            .compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed, guard)
            .map_err(|_| ())?;

        Ok(Some(unsafe {
            let data = ptr::read(&head_ref.data);
            guard.defer_destroy(head);
            ManuallyDrop::into_inner(data)
        }))
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.head.load(Ordering::Acquire, guard).is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        unsafe {
            let guard = unprotected();
            while let Ok(Some(_)) = self.try_pop(guard) {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread::scope;

    #[test]
    fn push() {
        let stack = TreiberStack::default();

        scope(|scope| {
            for _ in 0..10 {
                let _unused = scope.spawn(|| {
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
