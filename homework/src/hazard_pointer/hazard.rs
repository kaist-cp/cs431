use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use std::collections::HashSet;
use std::fmt;

#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};
#[cfg(feature = "check-loom")]
use loom::sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering};

use super::HAZARDS;

/// Represents the ownership of a hazard pointer slot.
pub struct Shield<T> {
    slot: NonNull<HazardSlot>,
    _marker: PhantomData<*const T>, // !Send + !Sync
}

impl<T> Shield<T> {
    /// Creates a new shield for hazard pointer.
    pub fn new(hazards: &HazardBag) -> Self {
        let slot = hazards.acquire_slot();
        Self {
            slot: slot.into(),
            _marker: PhantomData,
        }
    }

    /// Try protecting the pointer `*pointer`.
    /// 1. Store `*pointer` to the hazard slot.
    /// 2. Check if `src` still points to `*pointer` (validation) and update `pointer` to the
    ///    latest value.
    /// 3. If validated, return true. Otherwise, clear the slot (store 0) and return false.
    pub fn try_protect(&self, pointer: &mut *const T, src: &AtomicPtr<T>) -> bool {
        todo!()
    }

    /// Get a protected pointer from `src`.
    pub fn protect(&self, src: &AtomicPtr<T>) -> *const T {
        let mut pointer = src.load(Ordering::Relaxed) as *const T;
        while !self.try_protect(&mut pointer, src) {
            #[cfg(feature = "check-loom")]
            loom::sync::atomic::spin_loop_hint();
        }
        pointer
    }
}

impl<T> Default for Shield<T> {
    fn default() -> Self {
        Self::new(&HAZARDS)
    }
}

impl<T> Drop for Shield<T> {
    /// Clear and release the ownership of the hazard slot.
    fn drop(&mut self) {
        todo!()
    }
}

impl<T> fmt::Debug for Shield<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shield")
            .field("slot address", &self.slot)
            .field("slot data", unsafe { self.slot.as_ref() })
            .finish()
    }
}

/// Global bag (multiset) of hazards pointers.
/// - `HazardBag.head` and `HazardSlot.next` form a grow-only list of all hazard slots. Slots are
///   never removed from this list.
/// - `HazardBag.head_available` and `HazardSlot.next_available` from an "overlay" Treiber stack of
///   slots that are not owned by a shield. This is for efficient recycling of the slots.
///
/// The figure below describes the state of the hazard bag after creating slots 1-5, and then
/// releasing slots 4 and 2.
///
/// ```text
///       next  s5        s4        s3        s2        s1
/// head  ---> +--+ ---> +--+ ---> +--+ ---> +--+ ---> +--+
///            |--|      |--|      |--|      |--|      |--|
///            +--+      +--+      +--+      +--+      +--+
///                       ^                  |  ^
///                       |  next_available  |  |
///                       +------------------+  |
/// head_available -----------------------------+
/// ```
#[derive(Debug)]
pub struct HazardBag {
    head: AtomicPtr<HazardSlot>,
    head_available: AtomicPtr<HazardSlot>,
}

/// See `HazardBag`
#[derive(Debug)]
struct HazardSlot {
    root: NonNull<HazardBag>,
    // Machine representation of the hazard pointer.
    hazard: AtomicUsize,
    // Immutable pointer to the next slot in the bag.
    next: *const HazardSlot,
    // Pointer to the next node in the available slot stack.
    next_available: AtomicPtr<HazardSlot>,
}

impl HazardSlot {
    fn new(root: NonNull<HazardBag>) -> Self {
        Self {
            root,
            hazard: AtomicUsize::new(0),
            next: ptr::null(),
            next_available: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl HazardBag {
    #[cfg(not(feature = "check-loom"))]
    /// Creates a new global hazard set.
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            head_available: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[cfg(feature = "check-loom")]
    /// Creates a new global hazard set.
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            head_available: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Acquires a slot in the hazard set, either by taking an available slot or allocating a new
    /// slot.
    fn acquire_slot(&self) -> &HazardSlot {
        todo!()
    }

    /// Pops a slot from available slot stack, if any.
    fn pop_available(&self) -> Option<&HazardSlot> {
        todo!()
    }

    /// Push a released slot to the available slot stack.
    ///
    /// Safety: The slot must have been acquired from this hazard bag.
    unsafe fn push_available(&self, slot: &HazardSlot) {
        todo!()
    }

    /// Returns all the hazards in the set. The returned set must not contain 0.
    pub fn all_hazards(&self) -> HashSet<usize> {
        todo!()
    }
}

impl Drop for HazardBag {
    fn drop(&mut self) {
        todo!()
    }
}

unsafe impl Send for HazardSlot {}
unsafe impl Sync for HazardSlot {}

#[cfg(all(test, not(feature = "check-loom")))]
mod tests {
    use super::{HazardBag, Shield};
    use std::collections::HashSet;
    use std::mem;
    use std::ops::Range;
    use std::sync::{atomic::AtomicPtr, Arc};
    use std::thread;

    const THREADS: usize = 8;
    const VALUES: Range<usize> = 1..1024;

    // `all_hazards` should return hazards protected by shield(s).
    #[test]
    fn all_hazards_protected() {
        let hazard_bag = Arc::new(HazardBag::new());
        let _ = (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        shield.protect(&src);
                        // leak the shield so that
                        mem::forget(shield);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|th| th.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(hazard_bag.all_hazards(), VALUES.collect())
    }

    // `all_hazards` should not return values that are no longer protected.
    #[test]
    fn all_hazards_unprotected() {
        let hazard_bag = Arc::new(HazardBag::new());
        let _ = (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        shield.protect(&src);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|th| th.join().unwrap())
            .collect::<Vec<_>>();
        assert!(hazard_bag.all_hazards().is_empty())
    }

    // `acquire_slot` should recycle existing slots.
    #[test]
    fn recycle_slots() {
        let hazard_bag = HazardBag::new();
        // allocate slots
        let shields = (0..1024)
            .map(|_| Shield::<()>::new(&hazard_bag))
            .collect::<Vec<_>>();
        // slot addresses
        let old_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();
        // release the slots
        drop(shields);

        let shields = (0..128)
            .map(|_| Shield::<()>::new(&hazard_bag))
            .collect::<Vec<_>>();
        let new_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();

        // no new slots should've been created
        assert!(new_slots.is_subset(&old_slots));
    }
}
