use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::sleep;
use std::time::Duration;

use crossbeam_channel::bounded;
use cs431_homework::hello_server::ThreadPool;

const NUM_THREADS: usize = 4;
const NUM_JOBS: usize = 1024;

#[test]
fn thread_pool_parallel() {
    let pool = ThreadPool::new(NUM_THREADS);
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let (done_sender, done_receiver) = bounded(NUM_THREADS);
    for _ in 0..NUM_THREADS {
        let barrier = barrier.clone();
        let done_sender = done_sender.clone();
        pool.execute(move || {
            let _ = barrier.wait();
            done_sender.send(()).unwrap();
        });
    }
    for _ in 0..NUM_THREADS {
        done_receiver.recv_timeout(Duration::from_secs(3)).unwrap();
    }
}

// Run jobs that take NUM_JOBS milliseconds as a whole.
fn run_jobs(pool: &ThreadPool, counter: &Arc<AtomicUsize>) {
    for _ in 0..NUM_JOBS {
        let counter = counter.clone();
        pool.execute(move || {
            sleep(Duration::from_millis(NUM_THREADS as u64));
            let _ = counter.fetch_add(1, Ordering::Relaxed);
        });
    }
}

/// `join` blocks until all jobs are finished.
#[test]
fn thread_pool_join_block() {
    let pool = ThreadPool::new(NUM_THREADS);
    let counter = Arc::new(AtomicUsize::new(0));
    run_jobs(&pool, &counter);
    pool.join();
    assert_eq!(counter.load(Ordering::Relaxed), NUM_JOBS);
}

/// `drop` blocks until all jobs are finished.
#[test]
fn thread_pool_drop_block() {
    let pool = ThreadPool::new(NUM_THREADS);
    let counter = Arc::new(AtomicUsize::new(0));
    run_jobs(&pool, &counter);
    drop(pool);
    assert_eq!(counter.load(Ordering::Relaxed), NUM_JOBS);
}

/// This indirectly tests if the worker threads' `JoinHandle`s are joined when the pool is
/// dropped.
#[test]
#[should_panic]
fn thread_pool_drop_propagate_panic() {
    let pool = ThreadPool::new(NUM_THREADS);
    pool.execute(move || {
        panic!();
    });
}
