use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ops::{Deref, DerefMut};

use either::Either;
use itertools::Itertools;

const INDEX_SENTINEL: u8 = 0xffu8;

#[derive(Default, Debug, Clone, Copy)]
#[repr(align(8))]
pub struct NodeHeader {
    pub length: u8,
    key: [u8; NodeHeader::MAX_LENGTH],
}

struct NodeBase4<V> {
    indexes: [u8; 4],
    children: [NodeBox<V>; 4],
}

struct NodeBase16<V> {
    indexes: [u8; 16],
    children: [NodeBox<V>; 16],
}

struct NodeBase48<V> {
    indexes: [u8; 256],
    children: [NodeBox<V>; 48],
}

struct NodeBase256<V> {
    children: [NodeBox<V>; 256],
}

struct NodeBaseV<V> {
    inner: ManuallyDrop<V>,
}

pub trait NodeBase<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)>;

    fn lookup_mut(&mut self, key: u8) -> Option<(u8, &mut NodeBox<V>)> {
        self.lookup(key)
            .map(|(i, n)| (i, unsafe { &mut *(n as *const _ as *mut NodeBox<V>) }))
    }

    fn insert(&mut self, key: u8, node: NodeBox<V>) -> Result<u8, NodeBox<V>>;

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()>;

    fn update(&mut self, index: u8, node: NodeBox<V>) -> Result<NodeBox<V>, NodeBox<V>>;

    // TODO: implement iterator and use it in drop
}

#[derive(Default, Debug)]
pub struct NodeBox<V> {
    inner: usize,
    _marker: PhantomData<Box<V>>,
}

impl NodeHeader {
    const MAX_LENGTH: usize = 23;

    pub fn set_key(&mut self, key: &[u8]) {
        let length = key.len();
        self.length = length as u8;
        self.key[0..length].copy_from_slice(key);
    }

    pub fn shrink_key(&mut self, delta: u8) {
        for i in delta..self.length {
            self.key[(i - delta) as usize] = self.key[i as usize];
        }
        self.length -= delta;
    }

    #[inline]
    pub fn key(&self) -> &[u8] {
        &self.key[0..usize::from(self.length)]
    }
}

impl<V> NodeBase<V> for NodeBase4<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        izip!(self.indexes.iter(), self.children.iter())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
            .map(|(i, (_, c))| (i as u8, c))
    }

    fn insert(&mut self, key: u8, node: NodeBox<V>) -> Result<u8, NodeBox<V>> {
        unimplemented!()
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        let index = usize::from(index);
        if index >= 4 {
            return Err(());
        }

        unsafe {
            *self.indexes.get_unchecked_mut(index) = INDEX_SENTINEL;
            Ok(mem::replace(
                self.children.get_unchecked_mut(index),
                NodeBox::null(),
            ))
        }
    }

    fn update(&mut self, index: u8, node: NodeBox<V>) -> Result<NodeBox<V>, NodeBox<V>> {
        unimplemented!()
    }
}

impl<V> NodeBase<V> for NodeBase16<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        izip!(self.indexes.iter(), self.children.iter())
            .enumerate()
            .find(|(_, (k, _))| **k == key)
            .map(|(i, (_, c))| (i as u8, c))
    }

    fn insert(&mut self, key: u8, node: NodeBox<V>) -> Result<u8, NodeBox<V>> {
        unimplemented!()
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        let index = usize::from(index);
        if index >= 4 {
            return Err(());
        }

        unsafe {
            *self.indexes.get_unchecked_mut(index) = INDEX_SENTINEL;
            Ok(mem::replace(
                self.children.get_unchecked_mut(index),
                NodeBox::null(),
            ))
        }
    }

    fn update(&mut self, index: u8, node: NodeBox<V>) -> Result<NodeBox<V>, NodeBox<V>> {
        unimplemented!()
    }
}

impl<V> NodeBase<V> for NodeBase48<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        let index = *unsafe { self.indexes.get_unchecked(usize::from(key)) };

        if index == INDEX_SENTINEL {
            return None;
        }

        Some((key, unsafe {
            self.children.get_unchecked(usize::from(index))
        }))
    }

    fn insert(&mut self, key: u8, node: NodeBox<V>) -> Result<u8, NodeBox<V>> {
        unimplemented!()
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        unsafe {
            let index = self.indexes.get_unchecked_mut(usize::from(index));
            *index = INDEX_SENTINEL;
            Ok(mem::replace(
                self.children.get_unchecked_mut(usize::from(*index)),
                NodeBox::null(),
            ))
        }
    }

    fn update(&mut self, index: u8, node: NodeBox<V>) -> Result<NodeBox<V>, NodeBox<V>> {
        unimplemented!()
    }
}

impl<V> NodeBase<V> for NodeBase256<V> {
    fn lookup(&self, key: u8) -> Option<(u8, &NodeBox<V>)> {
        Some((key, unsafe {
            self.children.get_unchecked(usize::from(key))
        }))
    }

    fn insert(&mut self, key: u8, node: NodeBox<V>) -> Result<u8, NodeBox<V>> {
        unimplemented!()
    }

    fn delete(&mut self, index: u8) -> Result<NodeBox<V>, ()> {
        unsafe {
            Ok(mem::replace(
                self.children.get_unchecked_mut(usize::from(index)),
                NodeBox::null(),
            ))
        }
    }

    fn update(&mut self, index: u8, node: NodeBox<V>) -> Result<NodeBox<V>, NodeBox<V>> {
        unimplemented!()
    }
}

impl<V> Deref for NodeBaseV<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V> DerefMut for NodeBaseV<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

const TAG_MASK: usize = 0b111;

// TODO: make sure alignment requirements with `CachePadded`
const_assert!(nodeheader_align; mem::align_of::<NodeHeader>() >= 8);
const_assert!(node4_size; mem::size_of::<(NodeHeader, NodeBase4<usize>)>() == 64);

impl<V> NodeBox<V> {
    #[inline]
    fn new_inner<T>(tag: usize, t: T) -> NodeBox<V> {
        let ptr = Box::into_raw(Box::new((NodeHeader::default(), t)));
        Self {
            inner: ptr as usize | tag,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn new_inner_default<T: Default>(tag: usize) -> NodeBox<V> {
        Self::new_inner(tag, T::default)
    }

    #[inline]
    unsafe fn drop_inner<T>(ptr: usize) {
        drop(Box::from_raw(ptr as *mut (NodeHeader, T)));
    }

    pub fn new4() -> NodeBox<V> {
        Self::new_inner_default::<NodeBase4<V>>(0)
    }

    pub fn new16() -> NodeBox<V> {
        Self::new_inner_default::<NodeBase16<V>>(1)
    }

    pub fn new48() -> NodeBox<V> {
        Self::new_inner_default::<NodeBase48<V>>(2)
    }

    pub fn new256() -> NodeBox<V> {
        Self::new_inner_default::<NodeBase256<V>>(3)
    }

    fn newv(inner: V) -> NodeBox<V> {
        Self::new_inner::<NodeBaseV<V>>(
            4,
            NodeBaseV::<V> {
                inner: ManuallyDrop::new(inner),
            },
        )
    }

    pub fn new_path<I, F>(key: I, f: F) -> (Self, *const V)
    where
        I: Iterator<Item = u8> + DoubleEndedIterator,
        F: FnOnce() -> V,
    {
        let mut node = NodeBox::newv(f());
        let result = node.deref_mut().unwrap().1.right().unwrap() as *const V;
        for chunk in key.rev().chunks(NodeHeader::MAX_LENGTH).into_iter() {
            let mut parent = NodeBox::new4();
            let (parent_header, parent_base) = parent.deref_mut().unwrap();
            let parent_base = parent_base.left().unwrap();

            let mut key = chunk.collect::<Vec<_>>();
            key.reverse();
            parent_header.set_key(&key);
            parent_base
                .insert(*unsafe { key.get_unchecked(0) }, node)
                .map_err(|_| ())
                .unwrap();

            node = parent;
        }
        (node, result)
    }

    pub fn null() -> NodeBox<V> {
        Self {
            inner: 0,
            _marker: PhantomData,
        }
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
                0 => Self::drop_inner::<NodeBase4<V>>(ptr),
                1 => Self::drop_inner::<NodeBase16<V>>(ptr),
                2 => Self::drop_inner::<NodeBase48<V>>(ptr),
                3 => Self::drop_inner::<NodeBase256<V>>(ptr),
                4 => Self::drop_inner::<NodeBaseV<V>>(ptr),
                _ => unreachable!(),
            }
        }
    }
}

impl<V> NodeBox<V> {
    pub fn deref(&self) -> Option<(&NodeHeader, Either<&dyn NodeBase<V>, &V>)> {
        let ptr = self.inner & !TAG_MASK;
        if ptr == 0 {
            return None;
        }

        let tag = self.inner & TAG_MASK;
        Some(unsafe {
            match tag {
                0 => {
                    let node = &*(ptr as *const (NodeHeader, NodeBase4<V>));
                    (&node.0, Either::Left(&node.1))
                }
                1 => {
                    let node = &*(ptr as *const (NodeHeader, NodeBase16<V>));
                    (&node.0, Either::Left(&node.1))
                }
                2 => {
                    let node = &*(ptr as *const (NodeHeader, NodeBase48<V>));
                    (&node.0, Either::Left(&node.1))
                }
                3 => {
                    let node = &*(ptr as *const (NodeHeader, NodeBase256<V>));
                    (&node.0, Either::Left(&node.1))
                }
                4 => {
                    let node = &*(ptr as *const (NodeHeader, NodeBaseV<V>));
                    (&node.0, Either::Right(&node.1))
                }
                _ => unreachable!(),
            }
        })
    }

    pub fn deref_mut(&mut self) -> Option<(&mut NodeHeader, Either<&mut dyn NodeBase<V>, &mut V>)> {
        let ptr = self.inner & !TAG_MASK;
        if ptr == 0 {
            return None;
        }

        let tag = self.inner & TAG_MASK;
        Some(unsafe {
            match tag {
                0 => {
                    let node = &mut *(ptr as *mut (NodeHeader, NodeBase4<V>));
                    (&mut node.0, Either::Left(&mut node.1))
                }
                1 => {
                    let node = &mut *(ptr as *mut (NodeHeader, NodeBase16<V>));
                    (&mut node.0, Either::Left(&mut node.1))
                }
                2 => {
                    let node = &mut *(ptr as *mut (NodeHeader, NodeBase48<V>));
                    (&mut node.0, Either::Left(&mut node.1))
                }
                3 => {
                    let node = &mut *(ptr as *mut (NodeHeader, NodeBase256<V>));
                    (&mut node.0, Either::Left(&mut node.1))
                }
                4 => {
                    let node = &mut *(ptr as *mut (NodeHeader, NodeBaseV<V>));
                    (&mut node.0, Either::Right(&mut node.1))
                }
                _ => unreachable!(),
            }
        })
    }
}

impl<V> Default for NodeBase4<V> {
    fn default() -> Self {
        Self {
            indexes: [INDEX_SENTINEL; 4],
            children: [
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
            ],
        }
    }
}

impl<V> Default for NodeBase16<V> {
    fn default() -> Self {
        Self {
            indexes: [INDEX_SENTINEL; 16],
            children: [
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
            ],
        }
    }
}

impl<V> Default for NodeBase48<V> {
    fn default() -> Self {
        Self {
            indexes: [INDEX_SENTINEL; 256],
            children: [
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
            ],
        }
    }
}

impl<V> Default for NodeBase256<V> {
    fn default() -> Self {
        Self {
            children: [
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
                NodeBox::null(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        // TODO
        assert_eq!(2 + 2, 4);
    }
}
