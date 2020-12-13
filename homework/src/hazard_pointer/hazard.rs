use core::marker::PhantomData;
use core::ptr;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::thread::ThreadId;

#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering};
#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering};

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
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        LocalHazardsIter {
            hazards: self,
            occupied: self.occupied.load(Ordering::Acquire),
        }
    }
}

#[derive(Debug)]
struct LocalHazardsIter<'s> {
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
        let (data, _) = align::decompose_tag::<T>(self.data);
        &*(data as *const T)
    }

    /// Dereferences the shielded hazard pointer is the pointer is not null.
    ///
    /// # Safety
    ///
    /// The pointer should point to a valid object of type `T` and the protection should be
    /// `validate`d. Invocations of this method should be properly synchronized with the other
    /// accesses to the object in order to avoid data race.
    pub unsafe fn as_ref(&self) -> Option<&T> {
        let (data, _) = align::decompose_tag::<T>(self.data);
        if data == 0 {
            None
        } else {
            Some(&*(data as *const T))
        }
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

impl<'s, T> fmt::Debug for Shield<'s, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (raw, tag) = align::decompose_tag::<T>(self.data);
        f.debug_struct("Shield")
            .field("raw", &raw)
            .field("tag", &tag)
            .field("hazards", &(self.hazards as *const _))
            .field("index", &self.index)
            .finish()
    }
}

/// Maps `ThreadId`s to their `Hazards`.
///
/// Uses a hash table based on append-only lock-free linked list for simplicity. In practice, this
/// is implemented using a more useful and efficient lock-free data structure.
#[derive(Debug)]
pub struct Hazards {
    heads: [AtomicPtr<Node>; Self::BUCKETS],
}

#[derive(Debug)]
struct Node {
    next: AtomicPtr<Node>,
    tid: ThreadId,
    hazards: LocalHazards,
}

impl Hazards {
    const BUCKETS: usize = 13;

    #[cfg(not(feature = "check-loom"))]
    /// Returns the hazard array of the given thread.
    pub const fn new() -> Self {
        Self {
            heads: [
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
            ],
        }
    }

    #[cfg(feature = "check-loom")]
    /// Returns the hazard array of the given thread.
    pub fn new() -> Self {
        Self {
            heads: [
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
            ],
        }
    }
    pub fn get(&self, tid: ThreadId) -> &LocalHazards {
        let index = {
            let mut s = DefaultHasher::new();
            tid.hash(&mut s);
            (s.finish() as usize) % Self::BUCKETS
        };

        'start: loop {
            let mut prev = unsafe { self.heads.get_unchecked(index) };
            let mut cur = prev.load(Ordering::Acquire);
            loop {
                if cur.is_null() {
                    let new = Box::into_raw(Box::new(Node {
                        next: AtomicPtr::new(ptr::null_mut()),
                        tid,
                        hazards: LocalHazards::new(),
                    }));
                    if prev
                        .compare_exchange(cur, new, Ordering::Release, Ordering::Relaxed)
                        .is_ok()
                    {
                        return unsafe { &(*new).hazards };
                    }
                    unsafe { drop(Box::from_raw(new)) };
                    continue 'start;
                }
                let cur_ref = unsafe { &*cur };
                if cur_ref.tid == tid {
                    return &cur_ref.hazards;
                }
                prev = &cur_ref.next;
                cur = prev.load(Ordering::Acquire);
            }
        }
    }

    /// Returns all elements of `Hazards` for all threads. The tags are erased.
    pub fn all_hazards(&self) -> HashSet<usize> {
        let mut set = HashSet::new();

        for b in &self.heads {
            let mut cur = b.load(Ordering::Acquire);
            while let Some(cur_ref) = unsafe { cur.as_ref() } {
                set.extend(cur_ref.hazards.iter());
                cur = cur_ref.next.load(Ordering::Acquire);
            }
        }
        set
    }
}

impl Drop for Hazards {
    fn drop(&mut self) {
        for b in self.heads.iter() {
            let mut cur = b.load(Ordering::Relaxed);
            while !cur.is_null() {
                cur = unsafe { Box::from_raw(cur).next.load(Ordering::Relaxed) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::atomic::Owned;
    use super::{Hazards, LocalHazards, Shield};
    use std::collections::HashSet;
    use std::mem;
    use std::sync::Arc;
    use std::thread;

    // support at least 8 hazard slots
    #[test]
    fn local_hazards_8() {
        let shareds = (0..8)
            .map(|i| Owned::new(i).into_shared())
            .collect::<Vec<_>>();
        let hazards = LocalHazards::new();
        let shields = shareds
            .iter()
            .map(|&s| unsafe { Shield::new(s, &hazards).unwrap() })
            .collect::<Vec<_>>();
        let values = shields
            .iter()
            .map(|s| unsafe { *s.deref() })
            .collect::<HashSet<_>>();

        assert_eq!(values, (0..8).collect());
        assert_eq!(hazards.iter().collect::<Vec<_>>().len(), 8);
        shareds
            .into_iter()
            .for_each(|s| unsafe { drop(s.into_owned()) });
    }

    #[test]
    fn all_hazards() {
        let global_hazards = Arc::new(Hazards::new());
        let hazards = (0..(2 * Hazards::BUCKETS))
            .map(|i| {
                let global_hazards = global_hazards.clone();
                thread::spawn(move || {
                    let hazards = global_hazards.get(thread::current().id());
                    let shared = Owned::new(i).into_shared();
                    mem::forget(unsafe { Shield::new(shared, &hazards) });
                    let data = shared.into_usize();
                    unsafe { drop(shared.into_owned()) }
                    data
                })
                .join()
                .unwrap()
            })
            .collect::<HashSet<_>>();
        assert_eq!(hazards, global_hazards.all_hazards())
    }
}
