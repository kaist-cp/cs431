use crossbeam_epoch as epoch;
use cs492_concur_homework::{NonblockingConcurrentMap, NonblockingMap, SplitOrderedList};

pub mod map;

#[test]
pub fn smoke() {
    let list = SplitOrderedList::<usize>::new();

    let guard = epoch::pin();

    assert_eq!(list.insert(&37, 37, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), None);
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.insert(&42, 42, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.delete(&37, &guard), Ok(&37));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);

    assert_eq!(list.delete(&37, &guard), Err(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);
}

#[test]
fn stress_sequential() {
    const STEPS: usize = 4096;
    map::stress_concurrent_sequential::<
        usize,
        NonblockingConcurrentMap<_, _, SplitOrderedList<usize>>,
    >(STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096;
    map::stress_concurrent::<usize, NonblockingConcurrentMap<_, _, SplitOrderedList<usize>>>(
        THREADS, STEPS,
    );
}

#[test]
fn log_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 24;
    map::log_concurrent::<usize, NonblockingConcurrentMap<_, _, SplitOrderedList<usize>>>(
        THREADS, STEPS,
    );
}
