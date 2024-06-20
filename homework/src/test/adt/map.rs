//! Testing utilities for map types.

use core::fmt::Debug;
use core::hash::Hash;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::thread::scope;

use crossbeam_epoch::pin;
use rand::prelude::*;

use crate::test::RandGen;
use crate::ConcurrentMap;

/// Runs many operations in a single thread and tests if it works like a map data structure using
/// `std::collections::HashMap` as reference.
pub fn stress_sequential<
    K: Clone + Debug + Eq + Hash + RandGen,
    V: Clone + Debug + Eq + RandGen,
    M: Default + ConcurrentMap<K, V>,
>(
    steps: usize,
) {
    enum Ops {
        LookupSome,
        LookupNone,
        Insert,
        DeleteSome,
        DeleteNone,
    }
    const OPS: [Ops; 5] = [
        Ops::LookupSome,
        Ops::LookupNone,
        Ops::Insert,
        Ops::DeleteSome,
        Ops::DeleteNone,
    ];

    let mut rng = thread_rng();
    let map = M::default();
    let mut hashmap = HashMap::new();

    for i in 0..steps {
        let op = OPS.choose(&mut rng).unwrap();

        match op {
            Ops::LookupSome => {
                let Some(key) = hashmap.keys().choose(&mut rng) else {
                    continue;
                };

                println!("iteration {i}: lookup({key:?}) (existing)");

                assert_eq!(map.lookup(key, &pin()), hashmap.get(key));
            }
            Ops::LookupNone => {
                let key = K::rand_gen(&mut rng);
                let hmap_res = hashmap.get(&key);
                let non = if hmap_res.is_some() { "" } else { "non-" };

                println!("iteration {i}: lookup({key:?}) ({non}existing)");

                assert_eq!(map.lookup(&key, &pin()), hmap_res);
            }
            Ops::Insert => {
                let key = K::rand_gen(&mut rng);
                let value = V::rand_gen(&mut rng);

                println!("iteration {i}: insert({key:?}, {value:?})");

                let map_res = map.insert(key.clone(), value.clone(), &pin());
                let hmap_res = if let Entry::Vacant(e) = hashmap.entry(key) {
                    let _ = e.insert(value);
                    Ok(())
                } else {
                    Err(value)
                };
                assert_eq!(map_res, hmap_res);
            }
            Ops::DeleteSome => {
                let Some(key) = hashmap.keys().choose(&mut rng).cloned() else {
                    continue;
                };

                println!("iteration {i}: delete({key:?}) (existing)");

                assert_eq!(
                    map.delete(&key, &pin()).cloned(),
                    hashmap.remove(&key).ok_or(())
                );
            }
            Ops::DeleteNone => {
                let key = K::rand_gen(&mut rng);
                let hmap_res = hashmap.remove(&key).ok_or(());
                let non = if hmap_res.is_ok() { "" } else { "non-" };

                println!("iteration {i}: delete({key:?}) ({non}existing)");

                assert_eq!(map.delete(&key, &pin()).cloned(), hmap_res);
            }
        }
    }
}

/// Runs random lookup operations concurrently.
pub fn lookup_concurrent<
    K: Clone + Debug + Eq + Hash + RandGen + Sync,
    V: Clone + Debug + Eq + RandGen + Sync,
    M: Default + Sync + ConcurrentMap<K, V>,
>(
    threads: usize,
    steps: usize,
) {
    enum Ops {
        LookupSome,
        LookupNone,
    }
    const OPS: [Ops; 2] = [Ops::LookupSome, Ops::LookupNone];

    let mut rng = thread_rng();
    let map = M::default();
    let mut hashmap = HashMap::new();

    for _ in 0..steps {
        let key = K::rand_gen(&mut rng);
        let value = V::rand_gen(&mut rng);
        let _unused = map.insert(key.clone(), value.clone(), &pin());
        let _ = hashmap.entry(key).or_insert(value);
    }

    scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..threads {
            let handle = s.spawn(|| {
                let mut rng = thread_rng();
                for _ in 0..steps {
                    let op = OPS.choose(&mut rng).unwrap();

                    let key = match op {
                        Ops::LookupSome => hashmap.keys().choose(&mut rng).unwrap().clone(),
                        Ops::LookupNone => K::rand_gen(&mut rng),
                    };
                    assert_eq!(map.lookup(&key, &pin()), hashmap.get(&key));
                }
            });
            handles.push(handle);
        }
    });
}

/// Runs random insert operations concurrently.
pub fn insert_concurrent<
    K: Clone + Debug + Eq + Hash + RandGen,
    V: Clone + Debug + Eq + RandGen,
    M: Default + Sync + ConcurrentMap<K, V>,
>(
    threads: usize,
    steps: usize,
) {
    let map = M::default();

    scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..threads {
            let handle = s.spawn(|| {
                let mut rng = thread_rng();
                for _ in 0..steps {
                    let key = K::rand_gen(&mut rng);
                    let value = V::rand_gen(&mut rng);
                    if map.insert(key.clone(), value.clone(), &pin()).is_ok() {
                        assert_eq!(map.lookup(&key, &pin()).unwrap(), &value);
                    }
                }
            });
            handles.push(handle);
        }
    });
}

enum Ops {
    Lookup,
    Insert,
    Delete,
}
const OPS: [Ops; 3] = [Ops::Lookup, Ops::Insert, Ops::Delete];

#[derive(Clone)]
/// Successful operations are logged as `Some`. Failed operations are `None`.
///
/// We currently only make use of the `Some` variant for `result`.
enum Log<K, V> {
    Lookup { key: K, result: Option<V> },
    Insert { key: K, result: Option<V> },
    Delete { key: K, result: Option<V> },
}

impl<K, V> Log<K, V> {
    fn key(&self) -> &K {
        match self {
            Self::Lookup { key, .. } | Self::Insert { key, .. } | Self::Delete { key, .. } => key,
        }
    }
}

/// Randomly runs many operations concurrently.
pub fn stress_concurrent<
    K: Debug + Eq + RandGen,
    V: Debug + Eq + RandGen,
    M: Default + Sync + ConcurrentMap<K, V>,
>(
    threads: usize,
    steps: usize,
) {
    let map = M::default();

    scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..threads {
            let handle = s.spawn(|| {
                let mut rng = thread_rng();
                for _ in 0..steps {
                    let op = OPS.choose(&mut rng).unwrap();
                    let key = K::rand_gen(&mut rng);

                    match op {
                        Ops::Lookup => {
                            let _ = map.lookup(&key, &pin());
                        }
                        Ops::Insert => {
                            let _ = map.insert(key, V::rand_gen(&mut rng), &pin());
                        }
                        Ops::Delete => {
                            let _ = map.delete(&key, &pin());
                        }
                    }
                }
            });
            handles.push(handle);
        }
    });
}

fn assert_logs_consistent<K: Debug + Eq + Hash, V: Debug + Eq + Hash>(logs: &[Log<K, V>]) {
    let mut per_key_logs = HashMap::new();
    for l in logs {
        per_key_logs.entry(l.key()).or_insert(vec![]).push(l);
    }

    for (k, logs) in per_key_logs {
        let mut inserts = HashMap::new();
        let mut deletes = HashMap::new();

        for l in logs.iter() {
            match l {
                Log::Insert {
                    result: Some(v), ..
                } => *inserts.entry(v).or_insert(0) += 1,
                Log::Delete {
                    result: Some(v), ..
                } => *deletes.entry(v).or_insert(0) += 1,
                _ => (),
            }
        }

        for l in logs {
            if let Log::Lookup {
                result: Some(v), ..
            } = l
            {
                assert!(
                    inserts.contains_key(v),
                    "key: {k:?}, value: {v:?}, lookup success but not inserted."
                );
            }
        }

        for (v, d_count) in deletes {
            let i_count = inserts.get(v).copied().unwrap_or(0);
            assert!(
                i_count >= d_count,
                "key: {k:?}, value: {v:?}, inserted {i_count} times but deleted {d_count} times."
            );
        }
    }
}

/// Randomly runs many operations concurrently and logs the operations & results per thread. Then
/// checks the consistency of the log. For example, if the key `k` was successfully deleted twice,
/// then `k` must have been inserted at least twice.
pub fn log_concurrent<
    K: Clone + Debug + Eq + Hash + RandGen + Send,
    V: Clone + Debug + Eq + Hash + RandGen + Send,
    M: Default + Sync + ConcurrentMap<K, V>,
>(
    threads: usize,
    steps: usize,
) {
    let map = M::default();

    let logs = scope(|s| {
        let mut handles = Vec::new();

        for _ in 0..threads {
            let handle = s.spawn(|| {
                let mut rng = thread_rng();
                let mut logs = Vec::new();

                for _ in 0..steps {
                    let op = OPS.choose(&mut rng).unwrap();
                    let key = K::rand_gen(&mut rng);

                    match op {
                        Ops::Lookup => {
                            let result = map.lookup(&key, &pin()).cloned();
                            logs.push(Log::Lookup { key, result });
                        }
                        Ops::Insert => {
                            let value = V::rand_gen(&mut rng);
                            let result = map
                                .insert(key.clone(), value.clone(), &pin())
                                .ok()
                                .map(|_| value);
                            logs.push(Log::Insert { key, result });
                        }
                        Ops::Delete => {
                            let result = map.delete(&key, &pin()).cloned().ok();
                            logs.push(Log::Delete { key, result });
                        }
                    }
                }
                logs
            });
            handles.push(handle);
        }
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect::<Box<[_]>>()
    });

    assert_logs_consistent(&logs);
}
