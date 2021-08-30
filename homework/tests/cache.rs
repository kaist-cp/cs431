use crossbeam_channel::bounded;
use crossbeam_utils::thread::scope;
use cs431_homework::hello_server::Cache;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Barrier;
use std::time::Duration;

const NUM_THREADS: usize = 8;
const NUM_KEYS: usize = 128;

#[test]
fn cache_no_duplicate_sequential() {
    let cache = Cache::default();
    cache.get_or_insert_with(1, |_| 1);
    cache.get_or_insert_with(2, |_| 2);
    cache.get_or_insert_with(3, |_| 3);
    assert_eq!(cache.get_or_insert_with(1, |_| panic!()), 1);
    assert_eq!(cache.get_or_insert_with(2, |_| panic!()), 2);
    assert_eq!(cache.get_or_insert_with(3, |_| panic!()), 3);
}

#[test]
fn cache_no_duplicate_concurrent() {
    for _ in 0..8 {
        let cache = Cache::default();
        let barrier = Barrier::new(NUM_THREADS);
        // Count the number of times the computation is run.
        let num_compute = AtomicUsize::new(0);
        scope(|s| {
            for _ in 0..NUM_THREADS {
                s.spawn(|_| {
                    barrier.wait();
                    for key in 0..NUM_KEYS {
                        cache.get_or_insert_with(key, |k| {
                            num_compute.fetch_add(1, Ordering::Relaxed);
                            k
                        });
                    }
                });
            }
        })
        .unwrap();
        assert_eq!(num_compute.load(Ordering::Relaxed), NUM_KEYS);
    }
}

#[test]
fn cache_no_block_disjoint() {
    let cache = &Cache::default();

    scope(|s| {
        // T1 blocks while inserting 1.
        let (t1_quit_sender, t1_quit_receiver) = bounded(0);
        s.spawn(move |_| {
            cache.get_or_insert_with(1, |k| {
                // block T1
                t1_quit_receiver.recv().unwrap();
                k
            });
        });

        // T2 must not be blocked by T1 when inserting 2.
        let (t2_done_sender, t2_done_receiver) = bounded(0);
        s.spawn(move |_| {
            cache.get_or_insert_with(2, |k| k);
            t2_done_sender.send(()).unwrap();
        });

        // If T2 is blocked, then this will time out.
        t2_done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("Inserting a different key should not block");

        // clean up
        t1_quit_sender.send(()).unwrap();
    })
    .unwrap();
}

#[test]
fn cache_no_reader_block() {
    let cache = &Cache::default();

    scope(|s| {
        let (t1_quit_sender, t1_quit_receiver) = bounded(0);
        let (t3_done_sender, t3_done_receiver) = bounded(0);

        // T1 blocks while inserting 1.
        s.spawn(move |s| {
            cache.get_or_insert_with(1, |k| {
                // T2 is blocked by T1 when reading 1
                s.spawn(move |_| cache.get_or_insert_with(1, |_| panic!()));

                // T3 should not be blocked when inserting 3.
                s.spawn(move |_| {
                    cache.get_or_insert_with(3, |k| k);
                    t3_done_sender.send(()).unwrap();
                });

                // block T1
                t1_quit_receiver.recv().unwrap();
                k
            });
        });

        // If T3 is blocked, then this will time out.
        t3_done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("Inserting a different key should not block");

        // clean up
        t1_quit_sender.send(()).unwrap();
    })
    .unwrap();
}
