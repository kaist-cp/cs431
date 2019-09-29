use core::fmt::Debug;
use core::iter::Peekable;

use crate::map::*;
use crate::node::*;

/// Adaptive radix tree.
#[derive(Debug)]
pub struct Art<V> {
    root: NodeBox<V>,
}

#[derive(Debug)]
struct Cursor<'a, V> {
    depth: usize,
    parent: &'a mut NodeBox<V>,
    child: &'a mut NodeBox<V>,
    index: u8,
    length: u8,
}

/// Entry API for Art.
#[derive(Debug)]
pub struct Entry<'a, V, I: Iterator<Item = u8> + DoubleEndedIterator> {
    cursor: Cursor<'a, V>,
    key: Peekable<I>,
}

impl<'a, V, I: 'a + Iterator<Item = u8> + DoubleEndedIterator> Entry<'a, V, I> {
    /// TODO
    #[inline]
    pub fn or_insert_with<F>(self, f: F) -> Result<&'a mut V, (&'a mut V, F)>
    where
        F: FnOnce() -> V,
    {
        unimplemented!()
    }

    /// TODO
    pub fn or_insert(self, default: V) -> Result<&'a mut V, (&'a mut V, V)> {
        self.or_insert_with(|| default).map_err(|(v, f)| (v, f()))
    }

    /// TODO
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Some(v) = self.lookup() {
            f(v);
        }

        self
    }

    /// TODO
    pub fn delete(mut self) -> Result<V, ()> {
        unimplemented!()
    }

    /// TODO
    pub fn lookup(&mut self) -> Option<&mut V> {
        if self.key.peek().is_some() {
            return None;
        }

        let (header, base) = self.cursor.child.deref_mut().unwrap();
        assert_eq!(self.cursor.length, header.length);
        base.right()
    }
}

impl<V> Default for Art<V> {
    fn default() -> Self {
        Self {
            root: NodeBox::new256(),
        }
    }
}

impl<V> Art<V> {
    const UTF8_SENTINEL: u8 = 0xffu8;

    /// Encodes a given string into an array of `u8`. Appending a sentinel value (0xff) to make sure
    /// a string is not a prefix of another.
    fn encode_key<'a>(key: &'a str) -> impl 'a + Iterator<Item = u8> + DoubleEndedIterator {
        key.bytes().chain(vec![Self::UTF8_SENTINEL].into_iter())
    }

    /// Creates an adaptive radix tree.
    pub fn new() -> Self {
        Self::default()
    }

    fn cursor<'a, I>(&'a mut self, key: &mut I) -> Cursor<'a, V>
    where
        I: 'a + Iterator<Item = u8>,
    {
        unimplemented!()
    }

    /// Creates an entry.
    pub fn entry<'a, I>(&'a mut self, mut key: I) -> Entry<'a, V, I>
    where
        I: 'a + Iterator<Item = u8> + DoubleEndedIterator,
    {
        let cursor = self.cursor(&mut key);
        Entry {
            cursor,
            key: key.peekable(),
        }
    }
}

impl<V> SequentialMap<V> for Art<V> {
    fn insert<'a>(&'a mut self, key: &'a str, value: V) -> Result<&'a mut V, (&'a mut V, V)> {
        let key = Self::encode_key(key);
        self.entry(key).or_insert(value)
    }

    fn delete(&mut self, key: &str) -> Result<V, ()> {
        let key = Self::encode_key(key);
        self.entry(key).delete()
    }

    fn lookup<'a>(&'a self, key: &'a str) -> Option<&'a V> {
        let key = Self::encode_key(key);
        unsafe {
            (*(self as *const _ as *mut Self))
                .entry(key)
                .lookup()
                .map(|v| &*(v as *const V))
        }
    }
}
