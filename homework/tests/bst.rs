use cs492_concur_homework::{Bst, SequentialMap};

pub mod map;

#[test]
fn bst_smoke() {
    let mut bst = map::Sequentialize::<_, _, Bst<String, _>>::default();
    assert!(bst.insert(&String::from("aa"), 42).is_ok());
    assert!(bst.insert(&String::from("bb"), 37).is_ok());
    assert_eq!(bst.lookup(&String::from("bb")), Some(&37));
    assert_eq!(bst.delete(&String::from("aa")), Ok(42));
    assert_eq!(bst.delete(&String::from("aa")), Err(()));
}

#[test]
fn bst_stress() {
    const STEPS: usize = 4096;
    map::stress_concurrent_sequential::<String, Bst<String, usize>>(STEPS);
}

#[test]
fn bst_stress_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096;
    map::stress_concurrent::<String, Bst<String, usize>>(THREADS, STEPS);
}

#[test]
fn bst_log_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 12;
    map::log_concurrent::<String, Bst<String, usize>>(THREADS, STEPS);
}
