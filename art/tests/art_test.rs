use cs492_concur_art::{Art, SequentialMap};

#[test]
fn smoke() {
    let mut art = Art::new();
    assert!(art.insert("aa", 42).is_ok());
    assert!(art.insert("bb", 37).is_ok());
    assert_eq!(art.lookup("bb"), Some(&37));
    assert_eq!(art.delete("aa"), Ok(42));
    assert_eq!(art.delete("aa"), Err(()));
}
