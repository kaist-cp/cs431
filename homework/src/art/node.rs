use core::cmp;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ops::{Deref, DerefMut};

use crossbeam_utils::CachePadded;
use either::Either;

use arr_macro::arr;
use itertools::izip;
use static_assertions::const_assert;

/// The sentinel value for index.
pub const KEY_ENDMARK: u8 = 0xffu8;
pub const KEY_INVALID: u8 = 0xfeu8;

/// The header of a node.
#[derive(Default, Debug, Clone)]
pub struct NodeHeader {
    /// The length of the key fragment.
    length: u8,
    /// The key fragment of the node used for path compression optimization.
    key: [u8; NodeHeader::MAX_LENGTH],
}

/// The body of an internal node of capacity 4.
struct NodeBody4<V> {
    /// The key for each entry.
    keys: [u8; 4],

    /// The child for each entry.
    children: [NodeBox<V>; 4],
}

/// The body of an internal node of capacity 16.
struct NodeBody16<V> {
    /// The key for each entry.
    keys: [u8; 16],

    /// The child for each entry.
    children: [NodeBox<V>; 16],
}

/// The body of an internal node of capacity 48.
struct NodeBody48<V> {
    /// The entry index for each key.
    indexes: [u8; 256],

    /// The child for each entry.
    children: [NodeBox<V>; 48],
}

/// The body of an internal node of capacity 256.
struct NodeBody256<V> {
    /// The child for each key.
    children: [NodeBox<V>; 256],
}

/// The body of a leaf node containing a value of type `V`.
struct NodeBodyV<V> {
    /// The contained value.
    inner: ManuallyDrop<V>,
}

/// The trait for the body of an internal node.
pub trait NodeBodyI<V> {
    /// Lookups the child of `key`.
    ///
    /// Returns `Some((i, n))` if `n` is the child of `key` at the internal index `i`.
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)>;

    /// Lookups the child of `key` mutably.
    ///
    /// Returns `Some((i, n))` if `n` is the child of `key` at the internal index `i`.
    fn lookup_mut(&mut self, key: u8) -> Option<(u8, &mut NodeBox<V>)> {
        self.lookup(key)
            .map(|(i, n)| (i, unsafe { &mut *(n as *const _ as *mut NodeBox<V>) }))
    }

    /// Updates the child of `key` with `node`.
    ///
    /// Returns `Ok((i, n))` if `n` was the original child of `key` at the internal index `i` before
    /// update; `Err(node)` if `node` cannot be inserted due to capacity reasons.
    fn update(&mut self, key: u8, node: NodeBox<V>) -> Result<(u8, NodeBox<V>), NodeBox<V>>;

    /// Deletes the child at the internal `index` obtained from `lookup()`, `lookup_mut()`, or
    /// `update()`.
    ///
    /// Returns `Ok(n)` if `n` was the original child at the internal `index`; `Err(())` if there is
    /// no such a child.
    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()>;

    /// Extracts children and makes `self` empty.
    ///
    /// Returns children as a vector of pairs of index and node.
    fn extract_children(&mut self) -> Vec<(u8, NodeBox<V>)>;
}

/// An owning pointer to a node.
#[derive(Debug)]
pub struct NodeBox<V> {
    inner: usize,
    _marker: PhantomData<Box<V>>,
}

impl NodeHeader {
    const MAX_LENGTH: usize = 23;

    /// Creates a new header with the given key.
    ///
    /// Returns `Ok(header)` if a new header is created; `Err(())` if the key is not fit for a
    /// header.
    pub fn new(key: &[u8]) -> Result<Self, ()> {
        let length = key.len();
        if length > Self::MAX_LENGTH {
            return Err(());
        }

        let mut header = Self::default();
        header.length = length as u8;
        header.key[0..length].copy_from_slice(key);
        Ok(header)
    }

    /// Shrinks the key by `delta`.
    pub fn shrink_key(&mut self, delta: u8) {
        for i in delta..self.length {
            self.key[usize::from(i - delta)] = self.key[usize::from(i)];
        }
        self.length -= delta;
    }

    /// Returns the length of the given header's key.
    #[inline]
    pub fn length(&self) -> u8 {
        self.length
    }

    /// Returns the key of the given header.
    #[inline]
    pub fn key(&self) -> &[u8] {
        &self.key[0..usize::from(self.length)]
    }
}

impl<V> NodeBodyI<V> for NodeBody4<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        izip!(self.keys.iter(), self.children.iter())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
            .map(|(i, (_, c))| (i as u8, c))
    }

    fn update(&mut self, key: u8, node: NodeBox<V>) -> Result<(u8, NodeBox<V>), NodeBox<V>> {
        if let Some((i, (_, c))) = izip!(self.keys.iter(), self.children.iter_mut())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
        {
            let child = mem::replace(c, node);
            return Ok((i as u8, child));
        }

        if let Some((i, (k, c))) = izip!(self.keys.iter_mut(), self.children.iter_mut())
            .enumerate()
            .find(|(_, (k, _))| **k == KEY_INVALID)
        {
            *k = key;
            *c = node;
            return Ok((i as u8, NodeBox::null()));
        }

        Err(node)
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        let index = usize::from(index);
        if index >= 4 {
            return Err(());
        }

        unsafe {
            *self.keys.get_unchecked_mut(index) = KEY_INVALID;
            Ok(mem::replace(
                self.children.get_unchecked_mut(index),
                NodeBox::null(),
            ))
        }
    }

    fn extract_children(&mut self) -> Vec<(u8, NodeBox<V>)> {
        let mut result = vec![];
        for (i, c) in izip!(&mut self.keys, &mut self.children) {
            if *i != KEY_INVALID {
                let child = mem::replace(c, NodeBox::null());
                result.push((*i, child));
                *i = KEY_INVALID;
            }
        }
        result
    }
}

impl<V> NodeBodyI<V> for NodeBody16<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        izip!(self.keys.iter(), self.children.iter())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
            .map(|(i, (_, c))| (i as u8, c))
    }

    fn update(&mut self, key: u8, node: NodeBox<V>) -> Result<(u8, NodeBox<V>), NodeBox<V>> {
        if let Some((i, (_, c))) = izip!(self.keys.iter(), self.children.iter_mut())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
        {
            let child = mem::replace(c, node);
            return Ok((i as u8, child));
        }

        if let Some((i, (k, c))) = izip!(self.keys.iter_mut(), self.children.iter_mut())
            .enumerate()
            .find(|(_, (k, _))| **k == KEY_INVALID)
        {
            *k = key;
            *c = node;
            return Ok((i as u8, NodeBox::null()));
        }

        Err(node)
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        let index = usize::from(index);
        if index >= 16 {
            return Err(());
        }

        unsafe {
            *self.keys.get_unchecked_mut(index) = KEY_INVALID;
            Ok(mem::replace(
                self.children.get_unchecked_mut(index),
                NodeBox::null(),
            ))
        }
    }

    fn extract_children(&mut self) -> Vec<(u8, NodeBox<V>)> {
        let mut result = vec![];
        for (i, c) in izip!(&mut self.keys, &mut self.children) {
            if *i != KEY_INVALID {
                let child = mem::replace(c, NodeBox::null());
                result.push((*i, child));
                *i = KEY_INVALID;
            }
        }
        result
    }
}

impl<V> NodeBodyI<V> for NodeBody48<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        let index = *unsafe { self.indexes.get_unchecked(usize::from(key)) };

        if index == KEY_INVALID {
            return None;
        }

        Some((key, unsafe {
            self.children.get_unchecked(usize::from(index))
        }))
    }

    fn update(&mut self, key: u8, node: NodeBox<V>) -> Result<(u8, NodeBox<V>), NodeBox<V>> {
        let index = self.indexes.get_mut(usize::from(key)).unwrap();

        if *index != KEY_INVALID {
            let child = mem::replace(
                unsafe { self.children.get_unchecked_mut(usize::from(*index)) },
                node,
            );
            return Ok((*index, child));
        }

        if let Some((i, c)) = self
            .children
            .iter_mut()
            .enumerate()
            .find(|(_, c)| c.is_null())
        {
            *index = i as u8;
            *c = node;
            return Ok((key, NodeBox::null()));
        }

        Err(node)
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        unsafe {
            let index = mem::replace(
                self.indexes.get_unchecked_mut(usize::from(index)),
                KEY_INVALID,
            );
            Ok(mem::replace(
                self.children.get_unchecked_mut(usize::from(index)),
                NodeBox::null(),
            ))
        }
    }

    fn extract_children(&mut self) -> Vec<(u8, NodeBox<V>)> {
        let mut result = vec![];
        for (i, j) in self.indexes.iter_mut().enumerate() {
            if *j != KEY_INVALID {
                let child = mem::replace(
                    unsafe { self.children.get_unchecked_mut(usize::from(*j)) },
                    NodeBox::null(),
                );
                result.push((i as u8, child));
                *j = KEY_INVALID;
            }
        }
        result
    }
}

impl<V> NodeBodyI<V> for NodeBody256<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        let node = unsafe { self.children.get_unchecked(usize::from(key)) };
        if node.is_null() {
            None
        } else {
            Some((key, node))
        }
    }

    fn update(&mut self, key: u8, node: NodeBox<V>) -> Result<(u8, NodeBox<V>), NodeBox<V>> {
        let child = mem::replace(
            unsafe { self.children.get_unchecked_mut(usize::from(key)) },
            node,
        );
        Ok((key, child))
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        unsafe {
            Ok(mem::replace(
                self.children.get_unchecked_mut(usize::from(index)),
                NodeBox::null(),
            ))
        }
    }

    fn extract_children(&mut self) -> Vec<(u8, NodeBox<V>)> {
        let mut result = vec![];
        for (i, c) in self.children.iter_mut().enumerate() {
            if !c.is_null() {
                let child = mem::replace(c, NodeBox::null());
                result.push((i as u8, child));
            }
        }
        result
    }
}

impl<V> Deref for NodeBodyV<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V> DerefMut for NodeBodyV<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

const TAG_BITS: usize = 3;
const TAG_MASK: usize = (1 << TAG_BITS) - 1;
const_assert!(mem::align_of::<CachePadded<()>>() >= (1 << TAG_BITS));

impl<V> NodeBox<V> {
    #[inline]
    fn new_inner<T>(header: NodeHeader, tag: usize, t: T) -> NodeBox<V> {
        let ptr = Box::into_raw(Box::new(CachePadded::new((header, t))));
        Self {
            inner: ptr as usize | tag,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn new_inner_default<T: Default>(header: NodeHeader, tag: usize) -> NodeBox<V> {
        Self::new_inner(header, tag, T::default())
    }

    #[inline]
    unsafe fn drop_inner<T>(ptr: usize) {
        drop(Box::from_raw(ptr as *mut CachePadded<(NodeHeader, T)>));
    }

    /// Creates a new `NodeBox` with a given `header` and `children`. The size of the new node is at
    /// least as large as `min_size`.
    ///
    /// If more than one child is given for a key, then the last child will be inserted to the node;
    /// all the previous children are dropped.
    ///
    /// # Panics
    ///
    /// Panics if the number of `children` or `min_size` exceeds 256.
    pub fn newi(header: NodeHeader, children: Vec<(u8, NodeBox<V>)>, min_size: usize) -> Self {
        let size = cmp::max(children.len(), min_size);
        let mut node = if (0..=4).contains(&size) {
            Self::new_inner_default::<NodeBody4<V>>(header, 0)
        } else if (5..=16).contains(&size) {
            Self::new_inner_default::<NodeBody16<V>>(header, 1)
        } else if (17..=48).contains(&size) {
            Self::new_inner_default::<NodeBody48<V>>(header, 2)
        } else if (49..=256).contains(&size) {
            Self::new_inner_default::<NodeBody256<V>>(header, 3)
        } else {
            panic!("NodeBox::newi(): invalid size {}", size)
        };

        let base = node.deref_mut().unwrap().1.left().unwrap();
        for (i, c) in children.into_iter() {
            base.update(i, c).map_err(|_| ()).unwrap();
        }

        node
    }

    fn newv(header: NodeHeader, inner: V) -> NodeBox<V> {
        Self::new_inner::<NodeBodyV<V>>(
            header,
            4,
            NodeBodyV::<V> {
                inner: ManuallyDrop::new(inner),
            },
        )
    }

    /// Creates a path of `key` to a leaf node containing `f()`.
    ///
    /// Returns `(n, v)`, where `n` is the node representing the path and `v` is a pointer to the
    /// leaf node's value.
    pub fn new_path<I, F>(key: I, f: F) -> (Self, *const V)
    where
        I: Iterator<Item = u8> + DoubleEndedIterator,
        F: FnOnce() -> V,
    {
        let key = key.collect::<Vec<_>>();
        let mut chunks = key.rchunks(NodeHeader::MAX_LENGTH);
        let first_chunk = chunks.next().unwrap();

        let mut node = Self::newv(NodeHeader::new(first_chunk).unwrap(), f());
        let mut key = *unsafe { first_chunk.get_unchecked(0) };
        let result = node.deref_mut().unwrap().1.right().unwrap() as *const V;

        for chunk in chunks {
            let parent = Self::newi(NodeHeader::new(chunk).unwrap(), vec![(key, node)], 0);
            key = *unsafe { chunk.get_unchecked(0) };
            node = parent;
        }

        (node, result)
    }

    /// Creates a null `NodeBox`.
    pub fn null() -> Self {
        Self {
            inner: 0,
            _marker: PhantomData,
        }
    }

    /// Checks if the given `NodeBox` is null.
    pub fn is_null(&self) -> bool {
        self.inner == 0
    }

    /// Unboxes `self` and returns its containing value.
    ///
    /// # Panics
    ///
    /// Panics if it does not contain `NodeBodyV`.
    #[allow(dead_code)]
    pub fn into_value(self) -> V {
        let ptr = self.inner & !TAG_MASK;
        assert_eq!(self.inner & TAG_MASK, 4);

        let node = unsafe { Box::from_raw(ptr as *mut CachePadded<(NodeHeader, NodeBodyV<V>)>) };
        mem::forget(self);

        let (_, body) = CachePadded::into_inner(*node);
        ManuallyDrop::into_inner(body.inner)
    }
}

impl<V> Into<(NodeHeader, Vec<(u8, NodeBox<V>)>)> for NodeBox<V> {
    fn into(mut self) -> (NodeHeader, Vec<(u8, NodeBox<V>)>) {
        let (header, base) = self.deref_mut().unwrap();
        let header = header.clone();
        let base = base.left().unwrap().extract_children();
        drop(self);
        (header, base)
    }
}

impl<V> Drop for NodeBox<V> {
    fn drop(&mut self) {
        let ptr = self.inner & !TAG_MASK;
        if ptr == 0 {
            return;
        }

        let tag = self.inner & TAG_MASK;
        unsafe {
            match tag {
                0 => Self::drop_inner::<NodeBody4<V>>(ptr),
                1 => Self::drop_inner::<NodeBody16<V>>(ptr),
                2 => Self::drop_inner::<NodeBody48<V>>(ptr),
                3 => Self::drop_inner::<NodeBody256<V>>(ptr),
                4 => Self::drop_inner::<NodeBodyV<V>>(ptr),
                _ => panic!("invalid tag {}", tag),
            }
        }
    }
}

impl<V> NodeBox<V> {
    /// Dereferences the given `NodeBox`.
    ///
    /// Returns `None` if the given `NodeBox` is null. Otherwise, returns `Some(header, b)` where
    /// `header` and `b` is the given box's header and body. The `b` is either `Left(body)`, if it's
    /// an internal node and `body` is its body, or `Right(value)`, if it's a leaf node and `value`
    /// is a reference to the leaf node's value.
    pub fn deref(&self) -> Option<(&NodeHeader, Either<&dyn NodeBodyI<V>, &V>)> {
        let ptr = self.inner & !TAG_MASK;
        if ptr == 0 {
            return None;
        }

        let tag = self.inner & TAG_MASK;
        Some(unsafe {
            match tag {
                0 => {
                    let node = &*(ptr as *const CachePadded<(NodeHeader, NodeBody4<V>)>);
                    (&node.0, Either::Left(&node.1))
                }
                1 => {
                    let node = &*(ptr as *const CachePadded<(NodeHeader, NodeBody16<V>)>);
                    (&node.0, Either::Left(&node.1))
                }
                2 => {
                    let node = &*(ptr as *const CachePadded<(NodeHeader, NodeBody48<V>)>);
                    (&node.0, Either::Left(&node.1))
                }
                3 => {
                    let node = &*(ptr as *const CachePadded<(NodeHeader, NodeBody256<V>)>);
                    (&node.0, Either::Left(&node.1))
                }
                4 => {
                    let node = &*(ptr as *const CachePadded<(NodeHeader, NodeBodyV<V>)>);
                    (&node.0, Either::Right(&node.1))
                }
                _ => unreachable!(),
            }
        })
    }

    /// Dereferences the given `NodeBox` mutably.
    ///
    /// See the comments for `Self::deref()`.
    pub fn deref_mut(
        &mut self,
    ) -> Option<(&mut NodeHeader, Either<&mut dyn NodeBodyI<V>, &mut V>)> {
        let ptr = self.inner & !TAG_MASK;
        if ptr == 0 {
            return None;
        }

        let tag = self.inner & TAG_MASK;
        Some(match tag {
            0 => {
                let node: &mut (_, _) =
                    unsafe { &mut *(ptr as *mut CachePadded<(NodeHeader, NodeBody4<V>)>) };
                (&mut node.0, Either::Left(&mut node.1))
            }
            1 => {
                let node: &mut (_, _) =
                    unsafe { &mut *(ptr as *mut CachePadded<(NodeHeader, NodeBody16<V>)>) };
                (&mut node.0, Either::Left(&mut node.1))
            }
            2 => {
                let node: &mut (_, _) =
                    unsafe { &mut *(ptr as *mut CachePadded<(NodeHeader, NodeBody48<V>)>) };
                (&mut node.0, Either::Left(&mut node.1))
            }
            3 => {
                let node: &mut (_, _) =
                    unsafe { &mut *(ptr as *mut CachePadded<(NodeHeader, NodeBody256<V>)>) };
                (&mut node.0, Either::Left(&mut node.1))
            }
            4 => {
                let node: &mut (_, _) =
                    unsafe { &mut *(ptr as *mut CachePadded<(NodeHeader, NodeBodyV<V>)>) };
                (&mut node.0, Either::Right(&mut node.1))
            }
            _ => unreachable!(),
        })
    }
}

impl<V> Default for NodeBody4<V> {
    fn default() -> Self {
        Self {
            keys: [KEY_INVALID; 4],
            children: arr![NodeBox::null(); 4],
        }
    }
}

impl<V> Default for NodeBody16<V> {
    fn default() -> Self {
        Self {
            keys: [KEY_INVALID; 16],
            children: arr![NodeBox::null(); 16],
        }
    }
}

impl<V> Default for NodeBody48<V> {
    fn default() -> Self {
        Self {
            indexes: [KEY_INVALID; 256],
            children: arr![NodeBox::null(); 48],
        }
    }
}

impl<V> Default for NodeBody256<V> {
    fn default() -> Self {
        Self {
            children: arr![NodeBox::null(); 256],
        }
    }
}
