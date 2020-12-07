use core::sync::atomic::Ordering::*;

use crossbeam_utils::thread::scope;
use cs492_concur_homework::hazard_pointer::{collect, protect, retire, Atomic, Owned};

#[test]
fn counter() {
    const THREADS: usize = 4;
    const ITER: usize = 1024 * 16;

    let count = Atomic::new(0usize);
    scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                for _ in 0..ITER {
                    let mut new = Owned::new(0);
                    let mut cur = count.load(Acquire);
                    loop {
                        let shield = protect(cur).unwrap();
                        let cur2 = count.load(Relaxed);
                        if !shield.validate(cur2) {
                            cur = cur2;
                            continue;
                        }
                        let value = unsafe { *shield.deref() };
                        *new = value + 1;
                        let new_shared = new.into_shared();
                        match count.compare_and_set(cur, new_shared, AcqRel, Acquire) {
                            Ok(_) => {
                                retire(cur);
                                break;
                            }
                            Err(c) => {
                                new = unsafe { new_shared.into_owned() };
                                cur = c;
                            }
                        }
                    }
                }
            });
        }
    })
    .unwrap();
    let cur = count.load(Acquire);
    // exclusive access
    assert_eq!(unsafe { *cur.deref() }, THREADS * ITER);
    retire(cur);
    collect();
}

// NOTE: more tests will be added soon™
