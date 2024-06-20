#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{AtomicPtr, Ordering::*};
use std::thread::{scope, sleep};
use std::time::Duration;

use cs431_homework::hazard_pointer::{collect, retire, Shield};
#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicPtr, Ordering::*};
use queue::Queue;
use stack::Stack;

#[test]
fn counter() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = AtomicPtr::new(Box::leak(Box::new(0usize)));
    scope(|s| {
        for _ in 0..THREADS {
            let _ = s.spawn(|| {
                for _ in 0..ITER {
                    let mut new = Box::new(0);
                    let shield = Shield::default();
                    loop {
                        let cur_ptr = shield.protect(&count);
                        let value = unsafe { *cur_ptr };
                        *new = value + 1;
                        let new_ptr = Box::leak(new);
                        if count
                            .compare_exchange(cur_ptr, new_ptr, AcqRel, Acquire)
                            .is_ok()
                        {
                            unsafe { retire(cur_ptr) };
                            break;
                        } else {
                            new = unsafe { Box::from_raw(new_ptr) };
                        }
                    }
                }
            });
        }
    });
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur }, THREADS * ITER);
    unsafe { retire(cur) };
}

// like `counter`, but trigger interesting interleaving using `sleep` and always call `collect`.
#[test]
fn counter_sleep() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = AtomicPtr::new(Box::leak(Box::new(0usize)));
    scope(|s| {
        for _ in 0..THREADS {
            let _ = s.spawn(|| {
                for _ in 0..ITER {
                    let mut new = Box::new(0);
                    let shield = Shield::default();
                    loop {
                        let cur_ptr = {
                            let mut cur = count.load(Relaxed);
                            while let Err(new) = shield.try_protect(cur, &count) {
                                sleep(Duration::from_micros(1));
                                cur = new;
                            }
                            cur
                        };
                        sleep(Duration::from_micros(1));
                        let value = unsafe { *cur_ptr };
                        *new = value + 1;
                        let new_ptr = Box::leak(new);
                        if count
                            .compare_exchange(cur_ptr, new_ptr, AcqRel, Acquire)
                            .is_ok()
                        {
                            unsafe { retire(cur_ptr) };
                            collect();
                            break;
                        } else {
                            new = unsafe { Box::from_raw(new_ptr) };
                        }
                    }
                }
            });
        }
    });
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur }, THREADS * ITER);
    unsafe { retire(cur) };
}

#[test]
fn stack() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 16;

    let stack = Stack::default();
    scope(|s| {
        for _ in 0..THREADS {
            let _ = s.spawn(|| {
                for i in 0..ITER {
                    stack.push(i);
                    assert!(stack.try_pop().is_some());
                    collect();
                }
            });
        }
    });
    assert!(stack.try_pop().is_none());
}

#[test]
fn queue() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 32;

    let queue = Queue::default();
    scope(|s| {
        for _ in 0..THREADS {
            let _ = s.spawn(|| {
                for i in 0..ITER {
                    queue.push(i);
                    assert!(queue.try_pop().is_some());
                    collect();
                }
            });
        }
    });
}

#[test]
fn stack_queue() {
    const THREADS: usize = 8;
    const ITER: usize = 1024 * 16;

    let stack = Stack::default();
    let queue = Queue::default();
    scope(|s| {
        for _ in 0..THREADS {
            let _ = s.spawn(|| {
                for i in 0..ITER {
                    stack.push(i);
                    queue.push(i);
                    let _ = stack.try_pop();
                    let _ = queue.try_pop();
                    collect();
                }
            });
        }
    });
    assert!(stack.try_pop().is_none());
}

mod sync {
    use core::ptr;

    use cs431_homework::hazard_pointer::*;
    use cs431_homework::test::loom::sync::atomic::Ordering::*;
    use cs431_homework::test::loom::sync::atomic::{AtomicPtr, AtomicUsize};
    use cs431_homework::test::loom::sync::Arc;
    use cs431_homework::test::loom::{model, thread};

    #[test]
    fn try_protect_collect_sync() {
        model(|| {
            let atomic = Arc::new(AtomicPtr::new(Box::leak(Box::new(123usize))));

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let local = atomic.load(Relaxed);
                    if local.is_null() {
                        return;
                    }
                    let shield = Shield::default();
                    if shield.try_protect(local, &atomic).is_ok() {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *local }, 123);
                    }
                })
            };

            // unlink, retire, and collect
            let local = atomic.load(Relaxed);
            atomic.store(ptr::null_mut(), Relaxed);
            unsafe { retire(local) };
            collect();
            th.join().unwrap();
        })
    }

    #[test]
    fn protect_collect_sync() {
        model(|| {
            let atomic = Arc::new(AtomicPtr::new(ptr::null_mut::<usize>()));

            let th = {
                let atomic = atomic.clone();
                thread::spawn(move || {
                    let shield = Shield::default();
                    let local = shield.protect(&atomic);
                    if !local.is_null() {
                        // safe to deref a valid pointer via a validated shield
                        assert_eq!(unsafe { *local }, 123);
                    }
                })
            };

            // link
            let local = Box::into_raw(Box::new(123));
            atomic.store(local, Release);

            // unlink, retire, and collect
            atomic.store(ptr::null_mut(), Relaxed);
            unsafe { retire(local) };
            collect();

            th.join().unwrap();
        })
    }

    // Above tests can't detect the absence of release-acquire between `Shield::drop` and `collect`
    // for an unknown reason. So explicitly check release-acquire between `Shield::drop` and
    // `all_hazards`.
    #[test]
    fn shield_drop_all_hazards_sync() {
        model(|| {
            let obj = Box::into_raw(Box::new(AtomicUsize::new(0)));
            let atomic = Arc::new(AtomicPtr::new(obj));
            let obj = obj as usize;
            let shield = Shield::default();
            let local = shield.protect(&atomic);

            let th = {
                thread::spawn(move || {
                    let local = atomic.load(Relaxed);
                    if !HAZARDS.all_hazards().contains(&obj) {
                        assert_eq!(unsafe { (*local).load(Relaxed) }, 123);
                    }
                })
            };

            unsafe { (*local).store(123, Relaxed) };
            drop(shield);

            th.join().unwrap();
            drop(unsafe { Box::from_raw(local) });
        })
    }
}

mod stack {
    use core::mem::MaybeUninit;
    use core::ptr;
    #[cfg(not(feature = "check-loom"))]
    use core::sync::atomic::{AtomicPtr, Ordering::*};

    use cs431_homework::hazard_pointer::{retire, Shield};
    #[cfg(feature = "check-loom")]
    use loom::sync::atomic::{AtomicPtr, Ordering::*};

    /// Treiber's lock-free stack.
    #[derive(Debug, Default)]
    pub struct Stack<T> {
        head: AtomicPtr<Node<T>>,
    }

    #[derive(Debug)]
    struct Node<T> {
        data: MaybeUninit<T>,
        next: *mut Node<T>,
    }

    unsafe impl<T: Send> Send for Node<T> {}
    unsafe impl<T: Sync> Sync for Node<T> {}

    impl<T> Stack<T> {
        pub fn push(&self, t: T) {
            let new = Box::leak(Box::new(Node {
                data: MaybeUninit::new(t),
                next: ptr::null_mut(),
            }));

            let mut head = self.head.load(Relaxed);

            loop {
                new.next = head;

                match self.head.compare_exchange(head, new, Release, Relaxed) {
                    Ok(_) => break,
                    Err(current) => head = current,
                }
            }
        }

        pub fn try_pop(&self) -> Option<T> {
            let shield = Shield::default();
            loop {
                let head_ptr = shield.protect(&self.head);
                let head_ref = unsafe { head_ptr.as_ref() }?;

                if self
                    .head
                    .compare_exchange(head_ptr, head_ref.next, Relaxed, Relaxed)
                    .is_ok()
                {
                    let data = unsafe { head_ref.data.assume_init_read() };
                    unsafe { retire(head_ptr) };
                    return Some(data);
                }
            }
        }
    }

    impl<T> Drop for Stack<T> {
        fn drop(&mut self) {
            #[cfg(not(feature = "check-loom"))]
            let mut o_curr = *self.head.get_mut();
            #[cfg(feature = "check-loom")]
            let mut o_curr = self.head.load(Relaxed);

            while !o_curr.is_null() {
                let curr = unsafe { Box::from_raw(o_curr) };
                drop(unsafe { curr.data.assume_init() });
                o_curr = curr.next;
            }
        }
    }
}

mod queue {
    use core::mem::MaybeUninit;
    use core::ptr;
    #[cfg(not(feature = "check-loom"))]
    use core::sync::atomic::{AtomicPtr, Ordering::*};

    use cs431_homework::hazard_pointer::{retire, Shield};
    #[cfg(feature = "check-loom")]
    use loom::sync::atomic::{AtomicPtr, Ordering::*};

    /// Michael-Scott queue.
    #[derive(Debug)]
    pub struct Queue<T> {
        head: AtomicPtr<Node<T>>,
        tail: AtomicPtr<Node<T>>,
    }

    #[derive(Debug)]
    struct Node<T> {
        data: MaybeUninit<T>,
        next: AtomicPtr<Node<T>>,
    }

    unsafe impl<T: Send> Sync for Queue<T> {}
    unsafe impl<T: Send> Send for Queue<T> {}

    impl<T> Default for Queue<T> {
        fn default() -> Self {
            let sentinel = Box::leak(Box::new(Node {
                data: MaybeUninit::uninit(),
                next: AtomicPtr::default(),
            }));

            Self {
                head: AtomicPtr::new(sentinel),
                tail: AtomicPtr::new(sentinel),
            }
        }
    }

    impl<T> Queue<T> {
        pub fn push(&self, t: T) {
            let new = Box::leak(Box::new(Node {
                data: MaybeUninit::new(t),
                next: AtomicPtr::default(),
            }));
            let shield = Shield::default();

            loop {
                let tail = shield.protect(&self.tail);
                // SAFETY
                // 1. queue's `tail` is always valid as it will be CASed with valid nodes only.
                // 2. `tail` is protected & validated.
                let tail_ref = unsafe { &*tail };

                let next = tail_ref.next.load(Acquire);
                if !next.is_null() {
                    let _ = self.tail.compare_exchange(tail, next, Release, Relaxed);
                    continue;
                }

                if tail_ref
                    .next
                    .compare_exchange(ptr::null_mut(), new, Release, Relaxed)
                    .is_ok()
                {
                    let _ = self.tail.compare_exchange(tail, new, Release, Relaxed);
                    break;
                }
            }
        }

        pub fn try_pop(&self) -> Option<T> {
            let head_shield = Shield::default();
            let next_shield = Shield::default();
            let mut head = self.head.load(Acquire);
            loop {
                if let Err(new) = head_shield.try_protect(head, &self.head) {
                    head = new;
                    continue;
                }
                // SAFETY:
                // 1. queue's `head` is always valid as it will be CASed with valid nodes only.
                // 2. `head` is protected & validated.
                let head_ref = unsafe { &*head };

                let next = head_ref.next.load(Acquire);
                if next.is_null() {
                    return None;
                }
                next_shield.set(next);
                let next_ref = match Shield::validate(head, &self.head) {
                    Ok(_) => {
                        // SAFETY:
                        // 1. If `next` was not null, then it must be a valid node that another
                        //    thread has `push()`ed.
                        // 2. Validation: If `head` is not retired, then `next` is not retired. So
                        //    re-validating `head` also validates `next.
                        unsafe { &*next }
                    }
                    Err(new) => {
                        head = new;
                        continue;
                    }
                };

                let tail = self.tail.load(Relaxed);
                if tail == head {
                    let _ = self.tail.compare_exchange(tail, next, Release, Relaxed);
                }

                match self.head.compare_exchange(head, next, Release, Relaxed) {
                    Ok(_) => {
                        let result = unsafe { next_ref.data.assume_init_read() };
                        unsafe { retire(head) };
                        return Some(result);
                    }
                    Err(new) => head = new,
                }
            }
        }
    }

    impl<T> Drop for Queue<T> {
        fn drop(&mut self) {
            #[cfg(not(feature = "check-loom"))]
            let sentinel = unsafe { Box::from_raw(*self.head.get_mut()) };
            #[cfg(feature = "check-loom")]
            let sentinel = unsafe { Box::from_raw(self.head.load(Relaxed)) };

            let mut o_curr = sentinel.next.into_inner();
            while !o_curr.is_null() {
                let curr = unsafe { Box::from_raw(o_curr) };
                drop(unsafe { curr.data.assume_init() });
                o_curr = curr.next.into_inner();
            }
        }
    }
}
