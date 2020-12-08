use core::sync::atomic::Ordering::*;

use crossbeam_utils::thread::scope;
use cs492_concur_homework::hazard_pointer::{collect, get_protected, retire, Atomic, Owned};

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
                    loop {
                        let cur_shield = get_protected(&count).unwrap();
                        let value = unsafe { *cur_shield.deref() };
                        *new = value + 1;
                        let new_shared = new.into_shared();
                        if count
                            .compare_and_set(cur_shield.shared(), new_shared, AcqRel, Acquire)
                            .is_ok()
                        {
                            retire(cur_shield.shared());
                            break;
                        } else {
                            new = unsafe { new_shared.into_owned() };
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
}

// NOTE: more tests will be added soon™
