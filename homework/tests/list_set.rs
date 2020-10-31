use crossbeam_utils::thread;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};

use cs492_concur_homework::OrderedListSet;

#[test]
fn smoke() {
    let set = OrderedListSet::new();
    set.insert(1).unwrap();
    set.insert(2).unwrap();
    set.insert(3).unwrap();
    assert_eq!(set.remove(&2), Ok(2));
    for i in set.iter() {
        println!("{}", i);
    }
    assert_eq!(set.remove(&3), Ok(3));
}

#[test]
fn parallel_iter_end() {
    let set = OrderedListSet::new();
    set.insert(1).unwrap();
    set.insert(2).unwrap();
    let mut iter = set.iter();
    iter.next();
    iter.next();
    iter.next();
    thread::scope(|s| {
        s.spawn(|_| {
            // this shouldn't block
            let _ = set.iter().collect::<Vec<_>>();
        });
    })
    .unwrap();
    drop(iter);
}

#[test]
fn stress_sequential() {
    #[derive(Debug)]
    enum Ops {
        ContainsSome,
        ContainsNone,
        Insert,
        RemoveSome,
        RemoveNone,
        Iterate,
    }

    let ops = [
        Ops::ContainsSome,
        Ops::ContainsNone,
        Ops::Insert,
        Ops::RemoveSome,
        Ops::RemoveNone,
        Ops::Iterate,
    ];
    let mut rng = thread_rng();
    let set = OrderedListSet::default();
    let mut hashset = HashSet::<String>::new();

    const OPS: usize = 4096;

    for i in 0..OPS {
        let op = ops.choose(&mut rng).unwrap();

        match op {
            Ops::ContainsSome => {
                if let Some(key) = hashset.iter().choose(&mut rng) {
                    println!("iteration {}: contains({:?}) (existing)", i, key);
                    assert_eq!(set.contains(key), hashset.contains(key));
                }
            }
            Ops::ContainsNone => {
                let key = generate_random_string(&mut rng);
                println!("iteration {}: contains({:?}) (non-existing)", i, key);
                assert_eq!(set.contains(&key), hashset.contains(&key));
            }
            Ops::Insert => {
                let key = generate_random_string(&mut rng);
                println!("iteration {}: insert({:?})", i, key);
                assert_eq!(set.insert(key.clone()).is_ok(), hashset.insert(key));
            }
            Ops::RemoveSome => {
                let key = hashset.iter().choose(&mut rng).map(Clone::clone);
                if let Some(key) = key {
                    println!("iteration {}: remove({:?}) (existing)", i, key);
                    assert_eq!(set.remove(&key).is_ok(), hashset.remove(&key));
                }
            }
            Ops::RemoveNone => {
                let key = generate_random_string(&mut rng);
                println!("iteration {}: remove({:?}) (non-existing)", i, key);
                assert_eq!(set.remove(&key).is_ok(), hashset.remove(&key));
            }
            Ops::Iterate => {
                let result = set.iter().map(Clone::clone).collect::<HashSet<_>>();
                println!("iteration {}: iter() → {:?}", i, result);
                assert_eq!(result, hashset);
            }
        }
    }
}

const THREADS: usize = 16;
const STEPS: usize = 4096 * 8;

fn generate_random_string(rng: &mut ThreadRng) -> String {
    rng.sample_iter(&Alphanumeric).take(1).collect()
}

#[derive(Debug, Clone, Copy)]
enum Ops {
    Contains,
    Insert,
    Remove,
}

#[derive(Debug, Clone)]
enum Log {
    Contains { key: String, result: bool },
    Insert { key: String, result: bool },
    Remove { key: String, result: bool },
}

impl Log {
    fn key(&self) -> &String {
        match self {
            Self::Contains { key, .. } => key,
            Self::Insert { key, .. } => key,
            Self::Remove { key, .. } => key,
        }
    }
}

#[test]
fn stress_concurrent() {
    let ops = [Ops::Contains, Ops::Insert, Ops::Remove, Ops::Remove];

    let set = OrderedListSet::new();

    thread::scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                let mut rng = thread_rng();
                for _ in 0..STEPS {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Contains => {
                            let value = generate_random_string(&mut rng);
                            let _ = set.contains(&value);
                        }
                        Ops::Insert => {
                            let value = generate_random_string(&mut rng);
                            let _ = set.insert(value);
                        }
                        Ops::Remove => {
                            let value = generate_random_string(&mut rng);
                            let _ = set.remove(&value);
                        }
                    }
                }
            });
        }
    })
    .unwrap();
}

fn assert_logs_consistent(logs: &Vec<Vec<Log>>) {
    let mut per_key_logs = HashMap::<String, Vec<Log>>::new();
    for ls in logs {
        for l in ls {
            per_key_logs
                .entry(l.key().clone())
                .or_insert_with(|| Vec::new())
                .push(l.clone());
        }
    }

    for (k, logs) in &per_key_logs {
        let mut inserts = HashMap::<String, usize>::new();
        let mut deletes = HashMap::<String, usize>::new();

        for l in logs {
            match l {
                Log::Insert { result: true, .. } => *inserts.entry(k.clone()).or_insert(0) += 1,
                Log::Remove { result: true, .. } => *deletes.entry(k.clone()).or_insert(0) += 1,
                _ => (),
            }
        }

        for l in logs {
            match l {
                Log::Contains { key, result: true } => assert!(inserts.contains_key(key)),
                _ => (),
            }
        }

        for (k, v) in &deletes {
            assert!(inserts.get(k).unwrap() >= v);
        }
    }
}

#[test]
fn log_concurrent() {
    let ops = [Ops::Contains, Ops::Insert, Ops::Remove];

    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 12;

    let set = OrderedListSet::new();

    let logs = thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..THREADS {
            let handle = s.spawn(|_| {
                let mut rng = thread_rng();
                let mut logs = Vec::new();
                for _ in 0..STEPS {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Contains => {
                            let key = generate_random_string(&mut rng);
                            let result = set.contains(&key);
                            logs.push(Log::Contains {
                                key: key.clone(),
                                result,
                            });
                        }
                        Ops::Insert => {
                            let key = generate_random_string(&mut rng);
                            let result = set.insert(key.clone());
                            logs.push(Log::Insert {
                                key,
                                result: result.is_ok(),
                            });
                        }
                        Ops::Remove => {
                            let key = generate_random_string(&mut rng);
                            let result = set.remove(&key);
                            logs.push(Log::Remove {
                                key: key.clone(),
                                result: result.is_ok(),
                            });
                        }
                    }
                }
                logs
            });
            handles.push(handle);
        }
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    })
    .unwrap();

    assert_logs_consistent(&logs);
}

#[test]
fn iter_consistent() {
    const THREADS: usize = 15;
    const STEPS: usize = 4096 * 12;

    let set = OrderedListSet::new();

    // pre-fill with even numbers
    for i in (0..100).step_by(2).rev() {
        let _ = set.insert(i);
    }
    let evens = set.iter().copied().collect::<HashSet<_>>();

    let done = AtomicBool::new(false);
    thread::scope(|s| {
        // insert or remove odd numbers
        for _ in 0..THREADS {
            s.spawn(|_| {
                let mut rng = thread_rng();
                for _ in 0..STEPS {
                    let key = 2 * rng.gen_range(0, 50) + 1;
                    if rng.gen() {
                        let _ = set.insert(key);
                    } else {
                        let _ = set.remove(&key);
                    }
                }
                done.store(true, Release);
            });
        }
        // iterator consistency check
        s.spawn(|_| {
            while !done.load(Acquire) {
                let snapshot = set.iter().copied().collect::<Vec<_>>();
                // sorted
                assert!(snapshot.windows(2).all(|k| k[0] <= k[1]));
                // even numbers are not touched
                let snapshot = snapshot.into_iter().collect::<HashSet<_>>();
                assert!(evens.is_subset(&snapshot));
            }
        });
    })
    .unwrap();
}
