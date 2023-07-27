use core::iter::Peekable;
use core::ptr;

mod node;

use crate::map::*;
use node::*;

/// Adaptive radix tree.
///
/// IMPORTANT: you should not change this type.
#[derive(Debug)]
pub struct Art<V> {
    root: NodeBox<V>,
}

#[derive(Debug)]
struct Cursor<'a, V> {
    depth: usize,
    parent: Option<&'a mut NodeBox<V>>,
    child: &'a mut NodeBox<V>,
    index: u8,
    length: u8,
}

/// Entry API for Art.
///
/// See <https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html> for more details of the
/// entry API.
#[derive(Debug)]
pub struct Entry<'a, V, I: Iterator<Item = u8> + DoubleEndedIterator> {
    cursor: Cursor<'a, V>,
    key: Peekable<I>,
}

impl<'a, V, I: 'a + Iterator<Item = u8> + DoubleEndedIterator> Entry<'a, V, I> {
    /// Inserts the generated value if the entry is vacant.
    ///
    /// Returns `Ok(v)` if inserted, where `v` is a mutable reference to the inserted value;
    /// `Err((v, f))` if not inserted, where `v` is a mutable reference to the existing value and
    /// `f` is the given value generator.
    #[inline]
    pub fn or_insert_with<F>(mut self, f: F) -> Result<&'a mut V, (&'a mut V, F)>
    where
        F: FnOnce() -> V,
    {
        unimplemented!()
    }

    /// Inserts the given value if the entry is vacant.
    ///
    /// Returns `Ok(v)` if inserted, where `v` is a mutable reference to the inserted value;
    /// `Err((v, f))` if not inserted, where `v` is a mutable reference to the existing value and
    /// `f` is the given value generator.
    pub fn or_insert(self, default: V) -> Result<&'a mut V, (&'a mut V, V)> {
        self.or_insert_with(|| default).map_err(|(v, f)| (v, f()))
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the
    /// map.
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Some(v) = self.lookup() {
            f(v);
        }

        self
    }

    /// Deletes the value at the entry.
    ///
    /// Returns `Ok(v)` if the entry contains a value, `v`; `Err(())` if the entry does not contain
    /// a value.
    pub fn delete(mut self) -> Result<V, ()> {
        unimplemented!()
    }

    /// Lookups the entry's value.
    pub fn lookup(&mut self) -> Option<&mut V> {
        if self.key.peek().is_some() {
            return None;
        }

        let (header, body) = self.cursor.child.deref_mut().unwrap();
        assert_eq!(self.cursor.length, header.length());
        body.right()
    }
}

impl<V> Default for Art<V> {
    fn default() -> Self {
        Self {
            root: NodeBox::newi(NodeHeader::default(), vec![], 256),
        }
    }
}

impl<V> Art<V> {
    /// Encodes a given string into an array of `u8`. Appending a sentinel value (0xff) to make sure
    /// a string is not a prefix of another.
    fn encode_key(key: &'_ str) -> impl '_ + Iterator<Item = u8> + DoubleEndedIterator {
        key.bytes().chain(vec![KEY_ENDMARK])
    }

    /// Creates an adaptive radix tree.
    pub fn new() -> Self {
        Self::default()
    }

    fn cursor<'a, I>(&'a mut self, key: &mut Peekable<I>) -> Cursor<'a, V>
    where
        I: 'a + Iterator<Item = u8>,
    {
        unimplemented!()
    }

    /// Creates an entry.
    pub fn entry<'a, I>(&'a mut self, key: I) -> Entry<'a, V, I>
    where
        I: 'a + Iterator<Item = u8> + DoubleEndedIterator,
    {
        let mut key = key.peekable();
        let cursor = self.cursor(&mut key);
        Entry { cursor, key }
    }
}

impl<V> SequentialMap<str, V> for Art<V> {
    fn insert<'a>(&'a mut self, key: &'a str, value: V) -> Result<&'a mut V, (&'a mut V, V)> {
        let key = Self::encode_key(key);
        self.entry(key).or_insert(value)
    }

    fn delete(&mut self, key: &str) -> Result<V, ()> {
        let key = Self::encode_key(key);
        self.entry(key).delete()
    }

    fn lookup<'a>(&'a self, key: &'a str) -> Option<&'a V> {
        unimplemented!("Below code is UB.")
        // let key = Self::encode_key(key);
        // unsafe {
        //     (*(self as *const Self).cast_mut())
        //         .entry(key)
        //         .lookup()
        //         .map(|v| &*(v as *const V))
        // }
    }
}
