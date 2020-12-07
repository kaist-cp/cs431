use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::thread::ThreadId;

use super::align;
use super::atomic::Shared;

/// Per-thread array of hazard pointers.
///
/// Caveat: a thread may have up to 8 hazard pointers.
#[derive(Debug)]
pub struct LocalHazards {
    /// Bitmap that indicates the indices of occupied slots.
    occupied: AtomicU8,

    /// Array that contains the machine representation of hazard pointers without tag.
    elements: [AtomicUsize; 8],
}

impl Default for LocalHazards {
    fn default() -> Self {
        Self {
            occupied: Default::default(),
            elements: Default::default(),
        }
    }
}

impl LocalHazards {
    /// Creates a hazard pointer array.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a slot for a hazard pointer and returns its index. Returns `None` if the array is
    /// full.
    ///
    /// # Safety
    ///
    /// This function must be called only by the thread that owns this hazard array.
    pub unsafe fn alloc(&self, data: usize) -> Option<usize> {
        todo!()
    }

    /// Clears the hazard pointer at the given index.
    ///
    /// # Safety
    ///
    /// This function must be called only by the thread that owns this hazard array. The index must
    /// have been allocated.
    pub unsafe fn dealloc(&self, index: usize) {
        todo!()
    }

    /// Returns an iterator of hazard pointers (with tags erased).
    pub fn iter(&self) -> LocalHazardsIter<'_> {
        LocalHazardsIter {
            hazards: self,
            occupied: self.occupied.load(Ordering::Acquire),
        }
    }
}

#[derive(Debug)]
pub struct LocalHazardsIter<'s> {
    hazards: &'s LocalHazards,
    occupied: u8,
}

impl Iterator for LocalHazardsIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// Represents the ownership of a hazard pointer slot.
#[derive(Debug)]
pub struct Shield<'s, T> {
    data: usize, // preserves the tag of original `Shared`
    hazards: &'s LocalHazards,
    index: usize,
    _marker: PhantomData<&'s T>,
}

impl<'s, T> Shield<'s, T> {
    /// Creates a new shield for hazard pointer. Returns `None` if the hazard array is fully
    /// occupied.
    ///
    /// # Safety
    ///
    /// This function must be called only by the thread that owns this hazard array.
    pub unsafe fn new(pointer: Shared<T>, hazards: &'s LocalHazards) -> Option<Self> {
        todo!()
    }

    /// Returns `true` if the pointer is null.
    pub fn is_null(&self) -> bool {
        let (data, _) = align::decompose_tag::<T>(self.data);
        data == 0
    }

    /// Returns the `Shared` pointer protected by this shield. The original tag is preserved.
    pub fn shared(&self) -> Shared<T> {
        Shared::from_usize(self.data)
    }

    /// Dereferences the shielded hazard pointer.
    ///
    /// # Safety
    ///
    /// The pointer should point to a valid object of type `T` and the protection should be
    /// `validate`d. Invocations of this method should be properly synchronized with the other
    /// accesses to the object in order to avoid data race.
    pub unsafe fn deref(&self) -> &T {
        &*(self.data as *const T)
    }

    /// Check if `pointer` is protected by the shield. The tags are ignored.
    pub fn validate(&self, pointer: Shared<T>) -> bool {
        todo!()
    }
}

impl<'s, T> Drop for Shield<'s, T> {
    fn drop(&mut self) {
        todo!()
    }
}

/// Maps `ThreadId`s to their `Hazards`. In practice, this is implemented using a lock-free data
/// structures. However, we use a lock here in order to keep the homework simple.
pub struct Hazards(RwLock<HashMap<ThreadId, LocalHazards>>);

impl Hazards {
    /// Creates a new `Hazards`.
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    /// Returns the hazard of the given thread.
    pub fn get(&self, tid: ThreadId) -> &LocalHazards {
        let hazards = self.0.read().unwrap();
        if let Some(local_hazards) = hazards.get(&tid) {
            // safe because we don't delete or exclusively access the entry
            unsafe { &*(local_hazards as *const _) }
        } else {
            drop(hazards);
            let mut hazards = self.0.write().unwrap();
            unsafe { &*(hazards.entry(tid).or_insert_with(LocalHazards::new) as *const _) }
        }
    }

    /// Returns all elements of `Hazards` for all threads. The tags are erased.
    pub fn all_hazards(&self) -> HashSet<usize> {
        self.0
            .read()
            .unwrap()
            .values()
            .flat_map(LocalHazards::iter)
            .collect()
    }
}
