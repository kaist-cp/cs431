use core::marker::PhantomData;
use core::mem;
use cs492_concur_homework::{ConcurrentMap, SequentialMap};
use std::collections::HashMap;

use rand::distributions::Alphanumeric;
use rand::prelude::*;

use crossbeam_epoch::{pin, unprotected};
use crossbeam_utils::thread;

fn generate_random_string(rng: &mut ThreadRng) -> String {
    let length = rng.gen::<usize>() % 128;
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
                let key = generate_random_string(&mut rng);
                println!("iteration {}: lookup({:?}) (non-existing)", i, key);
                assert_eq!(
                    map.lookup(StringLike::string_as_ref(&key)),
                    hashmap.get(&key)
                );
            }
            Ops::Insert => {
                let key = generate_random_string(&mut rng);
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
                let key = generate_random_string(&mut rng);
                println!("iteration {}: delete({:?}) (non-existing)", i, key);
                assert_eq!(
                    map.delete(StringLike::string_as_ref(&key)),
                    hashmap.remove(&key).ok_or(())
                );
            }
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn stress_concurrent_sequential<
    S: StringLike + ?Sized,
    M: Default + ConcurrentMap<S, usize>,
>() {
    stress_sequential::<S, Sequentialize<S, usize, M>>();
}

pub fn stress_concurrent<S: StringLike + ?Sized, M: Default + Sync + ConcurrentMap<S, usize>>() {
    #[derive(Debug)]
    enum Ops {
        Lookup,
        Insert,
        Delete,
    }

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
                            let key = generate_random_string(&mut rng);
                            let _ = map.lookup(S::string_as_ref(&key), &pin(), |_v| {});
                        }
                        Ops::Insert => {
                            let key = generate_random_string(&mut rng);
                            let value = rng.gen::<usize>();
                            let _ = map.insert(S::string_as_ref(&key), value, &pin());
                        }
                        Ops::Delete => {
                            let key = generate_random_string(&mut rng);
                            let _ = map.delete(S::string_as_ref(&key), &pin());
                        }
                    }
                }
            });
        }
    })
    .unwrap();
}
