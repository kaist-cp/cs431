use cs492_concur_homework::Arc;
use std::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, Relaxed, SeqCst},
};
use std::sync::mpsc::channel;
use std::thread;

struct Canary(*mut AtomicUsize);

impl Drop for Canary {
    fn drop(&mut self) {
        unsafe {
            (*self.0).fetch_add(1, SeqCst);
        }
    }
}

#[test]
fn manually_share_arc() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let arc_v = Arc::new(v);

    let (tx, rx) = channel();

    let _t = thread::spawn(move || {
        let arc_v: Arc<Vec<i32>> = rx.recv().unwrap();
        assert_eq!((*arc_v)[3], 4);
    });

    tx.send(arc_v.clone()).unwrap();

    assert_eq!((*arc_v)[2], 3);
    assert_eq!((*arc_v)[4], 5);
}

#[test]
fn test_cowarc_clone_make_mut() {
    let mut cow0 = Arc::new(75);
    let mut cow1 = cow0.clone();
    let mut cow2 = cow1.clone();

    assert!(75 == *Arc::make_mut(&mut cow0));
    assert!(75 == *Arc::make_mut(&mut cow1));
    assert!(75 == *Arc::make_mut(&mut cow2));

    *Arc::make_mut(&mut cow0) += 1;
    *Arc::make_mut(&mut cow1) += 2;
    *Arc::make_mut(&mut cow2) += 3;

    assert!(76 == *cow0);
    assert!(77 == *cow1);
    assert!(78 == *cow2);

    // none should point to the same backing memory
    assert!(*cow0 != *cow1);
    assert!(*cow0 != *cow2);
    assert!(*cow1 != *cow2);
}

#[test]
fn test_cowarc_clone_unique2() {
    let mut cow0 = Arc::new(75);
    let cow1 = cow0.clone();
    let cow2 = cow1.clone();

    assert!(75 == *cow0);
    assert!(75 == *cow1);
    assert!(75 == *cow2);

    *Arc::make_mut(&mut cow0) += 1;
    assert!(76 == *cow0);
    assert!(75 == *cow1);
    assert!(75 == *cow2);

    // cow1 and cow2 should share the same contents
    // cow0 should have a unique reference
    assert!(*cow0 != *cow1);
    assert!(*cow0 != *cow2);
    assert!(*cow1 == *cow2);
}

#[test]
fn drop_arc() {
    let mut canary = AtomicUsize::new(0);
    let x = Arc::new(Canary(&mut canary as *mut AtomicUsize));
    let y = x.clone();
    drop(x);
    drop(y);
    assert!(canary.load(Acquire) == 1);
}

#[test]
fn test_count() {
    let a = Arc::new(0);
    assert!(Arc::count(&a) == 1);
    let b = a.clone();
    assert!(Arc::count(&a) == 2);
    assert!(Arc::count(&b) == 2);
}

#[test]
fn test_ptr_eq() {
    let five = Arc::new(5);
    let same_five = five.clone();
    let other_five = Arc::new(5);

    assert!(Arc::ptr_eq(&five, &same_five));
    assert!(!Arc::ptr_eq(&five, &other_five));
}

#[test]
fn test_stress() {
    let count = Arc::new(AtomicUsize::new(0));
    let handles = (0..8)
        .map(|_| {
            let count = count.clone();
            thread::spawn(move || {
                for _ in 0..128 {
                    count.fetch_add(1, Relaxed);
                }
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(count.load(Relaxed), 8 * 128);
}
