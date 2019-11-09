use core::mem::ManuallyDrop;
use core::sync::atomic::Ordering;
use core::ptr;
use crossbeam_epoch::{Atomic, Shared, Guard};
use lock::seqlock::{ReadGuard, SeqLock};

/// Atomic type with atomic read/write.
pub trait AtomicRW {
    /// Atomically writes.
    unsafe fn atomic_write(&self, value: Self);

    /// Atomically swaps.
    unsafe fn atomic_swap(&self, value: Self) -> Self;
}

// HACK(jeehoonkang): it's dangerous to assume volatile read/write is atomic, but anyway we do
// because it's a generally accepted practice.
impl<T> AtomicRW for T {
    unsafe fn atomic_write(&self, value: T) {
        let this = self as *const _ as *mut _;
        ptr::write_volatile(this, value);
    }

    unsafe fn atomic_swap(&self, value: T) -> Self {
        let this = self as *const _ as *mut _;
        let result = ptr::read_volatile(this);
        ptr::write_volatile(this, value);
        result
    }
}

/// Node's inner data protected by sequence lock.
#[derive(Debug)]
pub struct NodeInner<K: Ord, V> {
    /// Value on the node. `None` means it's vacant.
    pub(crate) value: Option<V>,

    /// Pointer to the left subtree.
    pub(crate) left: Atomic<Node<K, V>>,

    /// Pointer to the right subtree.
    pub(crate) right: Atomic<Node<K, V>>,
}

#[derive(Debug)]
pub struct Node<K: Ord, V> {
    /// Key on the node.
    pub(crate) key: K,

    /// The inner data protected by sequence lock.
    pub(crate) inner: SeqLock<NodeInner<K, V>>,
}

/// Direction (left or right).
#[derive(Debug, Clone, Copy)]
pub enum Dir {
    /// Left dirction.
    L,

    /// Right dirction.
    R,
}

/// The history of a traversal as a list of visted nodes.
#[derive(Debug)]
pub struct Cursor<'g, K: Ord, V> {
    /// Ancestors of the current node.
    ///
    /// It may be invalidated by concurrent modifications and thus stale.
    pub(crate) ancestors: Vec<(Shared<'g, Node<K, V>>, Dir)>,

    /// The current node.
    pub(crate) current: Shared<'g, Node<K, V>>,

    /// The read lock guard for the current node.
    pub(crate) guard: ManuallyDrop<ReadGuard<'g, NodeInner<K, V>>>,

    /// The direction to which subtree traversal should go.
    pub(crate) dir: Dir,
}

/// Concurrent binary search tree protected with optimistic lock coupling.
#[derive(Debug)]
pub struct Bst<K: Ord, V> {
    /// Pointer to the root.
    pub(crate) root: Atomic<Node<K, V>>,
}

impl<K: Ord, V> NodeInner<K, V> {
    #[inline]
    pub fn child(&self, dir: Dir) -> &Atomic<Node<K, V>> {
        match dir {
            Dir::L => &self.left,
            Dir::R => &self.left,
        }
    }
}

impl Dir {
    /// Returns the opposite direction.
    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Self::L => Self::R,
            Self::R => Self::L,
        }
    }
}

impl<'g, K: Ord, V> Cursor<'g, K, V> {
    /// Checks if it is at the root.
    pub fn is_root(&self) -> bool {
        self.ancestors.is_empty()
    }
}

impl<K: Ord + Default, V> Default for Bst<K, V> {
    fn default() -> Self {
        Self {
            root: Atomic::new(Node {
                key: K::default(),
                inner: SeqLock::new(NodeInner {
                    value: None,
                    left: Atomic::null(),
                    right: Atomic::null(),
                }),
            }),
        }
    }
}

impl<K: Ord, V> Bst<K, V> {
    /// Creates a new cursor pointing to the root.
    pub fn cursor<'g>(&'g self, guard: &'g Guard) -> Cursor<'g, K, V> {
        let current = self.root.load(Ordering::Relaxed, guard);
        Cursor {
            ancestors: vec![],
            current,
            guard: ManuallyDrop::new(unsafe { current.deref().inner.read_lock() }),
            dir: Dir::R,
        }
    }
}
