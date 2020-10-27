use cs492_concur_homework::{Art, SequentialMap, StrStringMap};

pub mod map;

#[test]
fn art_smoke() {
    let mut art = Art::new();
    assert!(art.insert("aa", 42).is_ok());
    assert!(art.insert("bb", 37).is_ok());
    assert_eq!(art.lookup("bb"), Some(&37));
    assert_eq!(art.delete("aa"), Ok(42));
    assert_eq!(art.delete("aa"), Err(()));
}

#[test]
fn art_regression_long_key() {
    let mut art = Art::<usize>::new();
    assert!(art
        .insert(
            "QRPnF2LvyOTg8CE2hg4bEHYQud6Y0igrypmOoLo6olwRmo6x4E4J9BVyo0LrmbjBagtVHVdL",
            10102680306753076321
        )
        .is_ok());
    assert_eq!(
        art.lookup("QRPnF2LvyOTg8CE2hg4bEHYQud6Y0igrypmOoLo6olwRmo6x4E4J9BVyo0LrmbjBagtVHVdL"),
        Some(&10102680306753076321)
    );
}

#[test]
fn art_regression_lookup_single_key() {
    let mut art = Art::<usize>::new();
    assert!(art.insert("AA", 2).is_ok());
    assert!(art.insert("AB", 2).is_ok());
    assert!(art.insert("A", 1).is_ok());
    assert_eq!(art.lookup("A"), Some(&1));
}

#[test]
fn art_regression_delete_after_enlarge() {
    let mut art = Art::<usize>::new();
    assert!(art.insert("AA", 2).is_ok());
    assert!(art.insert("AB", 3).is_ok());
    assert!(art.insert("AC", 3).is_ok());
    assert!(art.insert("AD", 3).is_ok());
    assert!(art.insert("A", 1).is_ok());
    assert!(art.delete("A").is_ok());
}

#[test]
fn art_stress() {
    map::stress_sequential::<String, StrStringMap<_, Art<usize>>>();
}
