use crossbeam_channel::bounded;
use crossbeam_epoch::pin;
use rand::prelude::*;
use std::collections::HashSet;
use std::iter::zip;
use std::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};
use std::thread;
use std::time::Duration;

use cs431_homework::test::adt::set;
use cs431_homework::{ConcurrentSet, OptimisticFineGrainedListSet};

#[test]
fn smoke() {
    let set = OptimisticFineGrainedListSet::new();
    assert!(set.insert(1));
    assert!(set.insert(2));
    assert!(set.insert(3));
    assert!(set.remove(&2));
    for (r, v) in zip(set.iter(&pin()), [1, 3]) {
        assert_eq!(*r.unwrap(), v);
    }
    assert!(set.remove(&3));
}

#[test]
fn read_no_block() {
    let set = &OptimisticFineGrainedListSet::new();
    assert!(set.insert(1));
    assert!(set.insert(2));

    let guard = pin();
    let mut iter = set.iter(&guard);
    assert_eq!(*iter.next().unwrap().unwrap(), 1);

    let (done_sender, done_receiver) = bounded(0);
    thread::scope(|s| {
        let _unused = s.spawn(move || {
            for v in 3..100 {
                set.insert(v);
            }
            done_sender.send(()).unwrap();
        });
        done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("Read should not block other operations");
    });
}

#[test]
fn iter_invalidate_end() {
    let set = &OptimisticFineGrainedListSet::new();
    assert!(set.insert(1));
    assert!(set.insert(2));
    let guard = pin();
    let mut iter = set.iter(&guard);
    iter.next();
    iter.next();
    assert!(set.insert(3));
    assert_eq!(iter.next(), Some(Err(())));
}

#[test]
fn stress_sequential() {
    const STEPS: usize = 4096;
    set::stress_sequential::<u8, OptimisticFineGrainedListSet<u8>>(STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 16;
    set::stress_concurrent::<u8, OptimisticFineGrainedListSet<u8>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 16;
    set::log_concurrent::<u8, OptimisticFineGrainedListSet<u8>>(THREADS, STEPS);
}

#[test]
fn iter_consistent() {
    const THREADS: usize = 15;
    const STEPS: usize = 4096 * 16;

    let set = OptimisticFineGrainedListSet::new();

    // pre-fill with even numbers
    for i in (0..100).step_by(2).rev() {
        assert!(set.insert(i));
    }
    let evens = set
        .iter(&pin())
        .map(|r| r.unwrap())
        .copied()
        .collect::<HashSet<_>>();

    let done = AtomicBool::new(false);
    thread::scope(|s| {
        // insert or remove odd numbers
        for _ in 0..THREADS {
            let _unused = s.spawn(|| {
                let mut rng = thread_rng();
                for _ in 0..STEPS {
                    let key = 2 * rng.gen_range(0..50) + 1;
                    if rng.gen() {
                        let _ = set.insert(key);
                    } else {
                        let _ = set.remove(&key);
                    }
                }
                done.store(true, Release);
            });
        }
        // iterator consistency check
        let _unused = s.spawn(|| {
            while !done.load(Acquire) {
                let mut snapshot = Vec::new();
                for r in set.iter(&pin()) {
                    match r {
                        Ok(&v) => snapshot.push(v),
                        Err(_) => break,
                    }
                }
                // sorted
                assert!(snapshot.windows(2).all(|k| k[0] <= k[1]));
                let max = snapshot.last().map(|&x| x).unwrap_or(0);
                let evens = evens
                    .iter()
                    .copied()
                    .filter(|&x| x <= max)
                    .collect::<HashSet<_>>();
                // even numbers are not touched
                let snapshot = snapshot.into_iter().collect::<HashSet<_>>();
                assert!(evens.is_subset(&snapshot));
            }
        });
    });
}
