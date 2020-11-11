use core::fmt;
use core::hash::Hash;
use core::marker::PhantomData;
use cs492_concur_homework::{ConcurrentMap, RandGen, SequentialMap};
use std::collections::HashMap;

use rand::prelude::*;

use crossbeam_epoch::pin;
use crossbeam_utils::thread;

pub fn stress_sequential<
    K: fmt::Debug + Clone + Eq + Hash + RandGen,
    M: Default + SequentialMap<K, usize>,
>(
    steps: usize,
) {
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
    let mut hashmap = HashMap::<K, usize>::new();

    for i in 0..steps {
        let op = ops.choose(&mut rng).unwrap();

        match op {
            Ops::LookupSome => {
                if let Some(key) = hashmap.keys().choose(&mut rng) {
                    println!("iteration {}: lookup({:?}) (existing)", i, key);
                    assert_eq!(map.lookup(key), hashmap.get(key));
                }
            }
            Ops::LookupNone => {
                let key = K::rand_gen(&mut rng);
                println!("iteration {}: lookup({:?}) (non-existing)", i, key);
                assert_eq!(map.lookup(&key), hashmap.get(&key));
            }
            Ops::Insert => {
                let key = K::rand_gen(&mut rng);
                let value = rng.gen::<usize>();
                println!("iteration {}: insert({:?}, {})", i, key, value);
                let _ = map.insert(&key, value);
                hashmap.entry(key).or_insert(value);
            }
            Ops::DeleteSome => {
                let key = hashmap.keys().choose(&mut rng).map(|k| k.clone());
                if let Some(key) = key {
                    println!("iteration {}: delete({:?}) (existing)", i, key);
                    assert_eq!(map.delete(&key), hashmap.remove(&key).ok_or(()));
                }
            }
            Ops::DeleteNone => {
                let key = K::rand_gen(&mut rng);
                println!("iteration {}: delete({:?}) (non-existing)", i, key);
                assert_eq!(map.delete(&key), hashmap.remove(&key).ok_or(()));
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
                .insert(key, value, &pin())
                .map(|_| &mut *hack)
                .map_err(|v| (&mut *hack, v))
        }
    }

    fn delete(&mut self, key: &K) -> Result<V, ()> {
        self.inner.delete(key, &pin())
    }

    fn lookup<'a>(&'a self, key: &'a K) -> Option<&'a V> {
        let ptr = self.inner.lookup(key, &pin(), |r| r.map(|v| v as *const _));
        ptr.map(|v| unsafe { &*v })
    }
}

pub fn stress_concurrent_sequential<
    K: fmt::Debug + Clone + Eq + Hash + RandGen,
    M: Default + ConcurrentMap<K, usize>,
>(
    steps: usize,
) {
    stress_sequential::<K, Sequentialize<K, usize, M>>(steps);
}

#[derive(Debug, Clone, Copy)]
enum Ops {
    Lookup,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
enum Log<K, V> {
    Lookup { key: K, value: Option<V> },
    Insert { key: K, value: Result<V, ()> },
    Delete { key: K, value: Result<V, ()> },
}

impl<K, V> Log<K, V> {
    fn key(&self) -> &K {
        match self {
            Self::Lookup { key, .. } => key,
            Self::Insert { key, .. } => key,
            Self::Delete { key, .. } => key,
        }
    }
}

pub fn stress_concurrent<
    K: fmt::Debug + Eq + Hash + RandGen,
    M: Default + Sync + ConcurrentMap<K, usize>,
>(
    threads: usize,
    steps: usize,
) {
    let ops = [Ops::Lookup, Ops::Insert, Ops::Delete];

    let map = M::default();

    thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|_| {
                let mut rng = thread_rng();
                for _ in 0..steps {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Lookup => {
                            let key = K::rand_gen(&mut rng);
                            let _ = map.lookup(&key, &pin(), |_v| {});
                        }
                        Ops::Insert => {
                            let key = K::rand_gen(&mut rng);
                            let value = rng.gen::<usize>();
                            let _ = map.insert(&key, value, &pin());
                        }
                        Ops::Delete => {
                            let key = K::rand_gen(&mut rng);
                            let _ = map.delete(&key, &pin());
                        }
                    }
                }
            });
        }
    })
    .unwrap();
}

fn assert_logs_consistent<K: Clone + Eq + Hash, V: Clone + Eq + Hash>(logs: &Vec<Vec<Log<K, V>>>) {
    let mut per_key_logs = HashMap::<K, Vec<Log<K, V>>>::new();
    for ls in logs {
        for l in ls {
            per_key_logs
                .entry(l.key().clone())
                .or_insert_with(|| Vec::new())
                .push(l.clone());
        }
    }

    for (_, logs) in &per_key_logs {
        let mut inserts = HashMap::<V, usize>::new();
        let mut deletes = HashMap::<V, usize>::new();

        for l in logs {
            match l {
                Log::Insert {
                    key: _,
                    value: Ok(v),
                } => *inserts.entry(v.clone()).or_insert(0) += 1,
                Log::Delete {
                    key: _,
                    value: Ok(v),
                } => *deletes.entry(v.clone()).or_insert(0) += 1,
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

pub fn log_concurrent<
    K: fmt::Debug + Clone + Eq + Hash + Send + RandGen,
    M: Default + Sync + ConcurrentMap<K, usize>,
>(
    threads: usize,
    steps: usize,
) {
    let ops = [Ops::Lookup, Ops::Insert, Ops::Delete];

    let map = M::default();

    let logs = thread::scope(|s| {
        let mut handles = Vec::new();
        for _ in 0..threads {
            let handle = s.spawn(|_| {
                let mut rng = thread_rng();
                let mut logs = Vec::new();
                for _ in 0..steps {
                    let op = ops.choose(&mut rng).unwrap();

                    match op {
                        Ops::Lookup => {
                            let key = K::rand_gen(&mut rng);
                            map.lookup(&key, &pin(), |value| {
                                logs.push(Log::Lookup {
                                    key: key.clone(),
                                    value: value.map(|v| *v),
                                });
                            });
                        }
                        Ops::Insert => {
                            let key = K::rand_gen(&mut rng);
                            let value = rng.gen::<usize>();
                            let result = map.insert(&key, value, &pin());
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
                            let key = K::rand_gen(&mut rng);
                            let result = map.delete(&key, &pin());
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
