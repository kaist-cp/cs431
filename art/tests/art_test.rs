use rand::distributions::Alphanumeric;
use rand::prelude::*;

use cs492_concur_art::{Art, SequentialMap};
use std::collections::HashMap;

#[derive(Debug)]
enum Ops {
    LookupSome,
    LookupNone,
    Insert,
    DeleteSome,
    DeleteNone,
}

fn generate_random_string(rng: &mut ThreadRng) -> String {
    let length = rng.gen::<usize>() % 128;
    rng.sample_iter(&Alphanumeric).take(length).collect()
}

#[test]
fn smoke() {
    let mut art = Art::new();
    assert!(art.insert("aa", 42).is_ok());
    assert!(art.insert("bb", 37).is_ok());
    assert_eq!(art.lookup("bb"), Some(&37));
    assert_eq!(art.delete("aa"), Ok(42));
    assert_eq!(art.delete("aa"), Err(()));
}

#[test]
fn regression_long_key() {
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
fn regression_lookup_single_key() {
    let mut art = Art::<usize>::new();
    assert!(art.insert("AA", 2).is_ok());
    assert!(art.insert("AB", 2).is_ok());
    assert!(art.insert("A", 1).is_ok());
    assert_eq!(art.lookup("A"), Some(&1));
}

#[test]
fn regression_delete_after_enlarge() {
    let mut art = Art::<usize>::new();
    assert!(art.insert("AA", 2).is_ok());
    assert!(art.insert("AB", 3).is_ok());
    assert!(art.insert("AC", 3).is_ok());
    assert!(art.insert("AD", 3).is_ok());
    assert!(art.insert("A", 1).is_ok());
    assert!(art.delete("A").is_ok());
}

#[test]
fn stress() {
    let ops = [
        Ops::LookupSome,
        Ops::LookupNone,
        Ops::Insert,
        Ops::DeleteSome,
        Ops::DeleteNone,
    ];
    let mut rng = thread_rng();
    let mut art = Art::new();
    let mut hashmap = HashMap::<String, usize>::new();

    const OPS: usize = 4096;

    for i in 0..OPS {
        let op = ops.choose(&mut rng).unwrap();

        match op {
            Ops::LookupSome => {
                if let Some(key) = hashmap.keys().choose(&mut rng) {
                    println!("iteration {}: lookup({:?}) (existing)", i, key);
                    assert_eq!(art.lookup(key), hashmap.get(key));
                }
            }
            Ops::LookupNone => {
                let key = generate_random_string(&mut rng);
                println!("iteration {}: lookup({:?}) (non-existing)", i, key);
                assert_eq!(art.lookup(&key), hashmap.get(&key));
            }
            Ops::Insert => {
                let key = generate_random_string(&mut rng);
                let value = rng.gen::<usize>();
                println!("iteration {}: insert({:?}, {})", i, key, value);
                let _ = art.insert(&key, value);
                hashmap.entry(key).or_insert(value);
            }
            Ops::DeleteSome => {
                let key = hashmap.keys().choose(&mut rng).map(|k| k.clone());
                if let Some(key) = key {
                    println!("iteration {}: delete({:?}) (existing)", i, key);
                    assert_eq!(art.delete(&key), hashmap.remove(&key).ok_or(()));
                }
            }
            Ops::DeleteNone => {
                let key = generate_random_string(&mut rng);
                println!("iteration {}: delete({:?}) (non-existing)", i, key);
                assert_eq!(art.delete(&key), hashmap.remove(&key).ok_or(()));
            }
        }
    }
}
