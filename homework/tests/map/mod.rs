use core::marker::PhantomData;
use cs492_concur_homework::{ConcurrentMap, SequentialMap};
use std::collections::HashMap;

use rand::distributions::Alphanumeric;
use rand::prelude::*;

use crossbeam_epoch::{pin, unprotected};
use crossbeam_utils::thread;

const SEQUENTIAL_KEY_MAX_LENGTH: usize = 128;
const CONCURRENT_KEY_MAX_LENGTH: usize = 2;

fn generate_random_string(rng: &mut ThreadRng, max_length: usize) -> String {
    let length = rng.gen::<usize>() % max_length;
    rng.sample_iter(&Alphanumeric).take(length).collect()
}

pub trait StringLike {
    fn as_ref_str(&self) -> &str;
    fn string_as_ref(s: &String) -> &Self;
}

impl StringLike for str {
    fn as_ref_str(&self) -> &str {
        self
    }

    fn string_as_ref(s: &String) -> &Self {
        s
    }
}

impl StringLike for String {
    fn as_ref_str(&self) -> &str {
        self
    }

    fn string_as_ref(s: &String) -> &Self {
        s
    }
}

pub fn stress_sequential<S: StringLike + ?Sized, M: Default + SequentialMap<S, usize>>() {
    #[derive(Debug)]
    enum Ops {
        LookupSome,
        LookupNone,
        Insert,
        DeleteSome,
        DeleteNone,
    }

    let ops = [
        Ops::LookupSome,
        Ops::LookupNone,
        Ops::Insert,
        Ops::DeleteSome,
        Ops::DeleteNone,
    ];
    let mut rng = thread_rng();
    let mut map = M::default();
    let mut hashmap = HashMap::<String, usize>::new();

    const OPS: usize = 4096;

    for i in 0..OPS {
        let op = ops.choose(&mut rng).unwrap();

        match op {
            Ops::LookupSome => {
                if let Some(key) = hashmap.keys().choose(&mut rng) {
                    println!("iteration {}: lookup({:?}) (existing)", i, key);
                    assert_eq!(map.lookup(StringLike::string_as_ref(key)), hashmap.get(key));
                }
            }
            Ops::LookupNone => {
                let key = generate_random_string(&mut rng, SEQUENTIAL_KEY_MAX_LENGTH);
                println!("iteration {}: lookup({:?}) (non-existing)", i, key);
                assert_eq!(
                    map.lookup(StringLike::string_as_ref(&key)),
                    hashmap.get(&key)
                );
            }
            Ops::Insert => {
                let key = generate_random_string(&mut rng, SEQUENTIAL_KEY_MAX_LENGTH);
                let value = rng.gen::<usize>();
                println!("iteration {}: insert({:?}, {})", i, key, value);
                let _ = map.insert(StringLike::string_as_ref(&key), value);
                hashmap.entry(key).or_insert(value);
            }
            Ops::DeleteSome => {
                let key = hashmap.keys().choose(&mut rng).map(|k| k.clone());
                if let Some(key) = key {
                    println!("iteration {}: delete({:?}) (existing)", i, key);
                    assert_eq!(
                        map.delete(StringLike::string_as_ref(&key)),
                        hashmap.remove(&key).ok_or(())
                    );
                }
            }
            Ops::DeleteNone => {
                let key = generate_random_string(&mut rng, SEQUENTIAL_KEY_MAX_LENGTH);
                println!("iteration {}: delete({:?}) (non-existing)", i, key);
                assert_eq!(
                    map.delete(StringLike::string_as_ref(&key)),
                    hashmap.remove(&key).ok_or(())
                );
            }
        }
    }
}

pub struct Sequentialize<K: ?Sized, V, M: ConcurrentMap<K, V>> {
    inner: M,
    _marker: PhantomData<(*const K, V)>,
}

impl<K: ?Sized, V, M: Default + ConcurrentMap<K, V>> Default for Sequentialize<K, V, M> {
    fn default() -> Self {
        Self {
            inner: M::default(),
            _marker: PhantomData,
        }
    }
}

impl<K: ?Sized, V, M: ConcurrentMap<K, V>> SequentialMap<K, V> for Sequentialize<K, V, M> {
    fn insert<'a>(&'a mut self, key: &'a K, value: V) -> Result<&'a mut V, (&'a mut V, V)> {
        unsafe {
            let hack = &value as *const _ as *mut V;
            self.inner
                .insert(key, value, unprotected())
                .map(|_| &mut *hack)
                .map_err(|v| (&mut *hack, v))
        }
    }

    fn delete(&mut self, key: &K) -> Result<V, ()> {
        self.inner.delete(key, unsafe { unprotected() })
    }

    fn lookup<'a>(&'a self, key: &'a K) -> Option<&'a V> {
        let ptr = self
            .inner
            .lookup(key, unsafe { unprotected() }, |r| r.map(|v| v as *const _));
        ptr.map(|v| unsafe { &*v })
    }
}

pub fn stress_concurrent_sequential<
    S: StringLike + ?Sized,
    M: Default + ConcurrentMap<S, usize>,
>() {
    stress_sequential::<S, Sequentialize<S, usize, M>>();
}

#[derive(Debug, Clone, Copy)]
enum Ops {
    Lookup,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
enum Log {
    Lookup {
        key: String,
        value: Option<usize>,
    },
    Insert {
        key: String,
        value: Result<usize, ()>,
    },
    Delete {
        key: String,
        value: Result<usize, ()>,
    },
}

impl Log {
    fn key(&self) -> &String {
        match self {
            Self::Lookup { key, .. } => key,
            Self::Insert { key, .. } => key,
            Self::Delete { key, .. } => key,
        }
    }
}

pub fn stress_concurrent<S: StringLike + ?Sized, M: Default + Sync + ConcurrentMap<S, usize>>() {
    let ops = [Ops::Lookup, Ops::Insert, Ops::Delete];

    const THREADS: usize = 16;
    const STEPS: usize = 4096;

    let map = M::default();

    thread::scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|_| {
                let mut rng = thread_rng();
                for _ in 0..STEPS {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Lookup => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            let _ = map.lookup(S::string_as_ref(&key), &pin(), |_v| {});
                        }
                        Ops::Insert => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            let value = rng.gen::<usize>();
                            let _ = map.insert(S::string_as_ref(&key), value, &pin());
                        }
                        Ops::Delete => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            let _ = map.delete(S::string_as_ref(&key), &pin());
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

    for (_, logs) in &per_key_logs {
        let mut inserts = HashMap::<usize, usize>::new();
        let mut deletes = HashMap::<usize, usize>::new();

        for l in logs {
            match l {
                Log::Insert {
                    key: _,
                    value: Ok(v),
                } => *inserts.entry(*v).or_insert(0) += 1,
                Log::Delete {
                    key: _,
                    value: Ok(v),
                } => *deletes.entry(*v).or_insert(0) += 1,
                _ => (),
            }
        }

        for l in logs {
            match l {
                Log::Lookup {
                    key: _,
                    value: Some(v),
                } => assert!(inserts.contains_key(v)),
                _ => (),
            }
        }

        for (k, v) in &deletes {
            assert!(inserts.get(k).unwrap() >= v);
        }
    }
}

pub fn log_concurrent<S: StringLike + ?Sized, M: Default + Sync + ConcurrentMap<S, usize>>() {
    let ops = [Ops::Lookup, Ops::Insert, Ops::Delete];

    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 12;

    let map = M::default();

    let logs = thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..THREADS {
            let handle = s.spawn(|_| {
                let mut rng = thread_rng();
                let mut logs = Vec::new();
                for _ in 0..STEPS {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Lookup => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            map.lookup(S::string_as_ref(&key), &pin(), |value| {
                                logs.push(Log::Lookup {
                                    key: key.clone(),
                                    value: value.map(|v| *v),
                                });
                            });
                        }
                        Ops::Insert => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            let value = rng.gen::<usize>();
                            let result = map.insert(S::string_as_ref(&key), value, &pin());
                            let value = match result {
                                Ok(()) => Ok(value),
                                Err(_) => Err(()),
                            };
                            logs.push(Log::Insert {
                                key: key.clone(),
                                value,
                            });
                        }
                        Ops::Delete => {
                            let key = generate_random_string(&mut rng, CONCURRENT_KEY_MAX_LENGTH);
                            let result = map.delete(S::string_as_ref(&key), &pin());
                            logs.push(Log::Delete {
                                key: key.clone(),
                                value: result,
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
