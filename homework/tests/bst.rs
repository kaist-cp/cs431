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
    map::stress_concurrent_sequential::<String, Bst<String, usize>>();
}

#[test]
fn bst_stress_concurrent() {
    map::stress_concurrent::<String, Bst<String, usize>>();
}

#[test]
fn bst_log_concurrent() {
    map::log_concurrent::<String, Bst<String, usize>>();
}
