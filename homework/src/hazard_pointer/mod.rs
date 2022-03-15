//! Hazard pointers.
//!
//! # Example
//!
//! ```
//! use std::ptr;
//! use std::sync::atomic::{AtomicPtr, Ordering};
//! use cs431_homework::hazard_pointer::{collect, retire, Shield};
//!
//! let shield = Shield::default();
//! let atomic = AtomicPtr::new(Box::leak(Box::new(1usize)));
//! let protected = shield.protect(&atomic);
//! assert_eq!(unsafe { *protected }, 1);
//!
//! // unlink the block and retire
//! atomic.store(ptr::null_mut(), Ordering::Relaxed);
//! unsafe { retire(protected); }
//!
//! // manually trigger reclamation (not necessary)
//! collect();
//! ```
//!
//! # Algorithm and Synchronization
//!
//! Suppose a data structure has a memory block b. T1 wants to read the value written in b and T2
//! wants to remove b from the data structure and free the memory. To prevent use-after-free, T1
//! has to ensure that b is not freed before reading b and T2 has to check that no other threads
//! are accessing b before freeing b. The hazard pointer library implements this mechanism as
//! follows:
//!
//! ```text
//! (T1-1) add b to the hazard list (Shield::new())   | (T2-1) unlink b (and retire b)
//! (T1-2) check if b is still reachable (validate()) | (T2-2) check if b is in the hazard list
//!        if so, deref b                             |        if not, free b
//! (T1-3) remove b from the hazard list              |
//!        (Shield::drop)                             |
//! ```
//!
//! To show that the algorithm prevents use-after-free in sequentially consistent memory model,
//! let's consider all possible interleavings of each step.
//!
//! First, if `T1-3 → T2-2` (`T2-2` is executed after `T1-3`), then `b` is freed after all
//! accesses.
//!
//! Second, in all of the remaining cases, either `T1-1 → T2-2` or `T2-1 → T1-2` holds (otherwise,
//! there is a cycle).
//! - If `T1-1 → T2-2`, then `b` is not freed.
//! - If `T2-1 → T1-2`, then the validation fails, so `T1` will not deref `b`.
//!
//! Therefore the algorithm is correct in sequentially consistent memory model. However, this is not
//! true in the relaxed memory model (construction of a counterexample is left as an exercise). The
//! problem is that in the relaxed memory model, `→` doesn't imply that the latter instruction
//! sees the effect of the earlier instruction. To fix this, we should add some synchronization
//! operations so that `→` implies that the effect of earlier instructions is visible to the later
//! instructions i.e. view inclusion. Let `I1 @ T1 ⊑ I2 @ T2` denote "T1's view before executing `I1` is
//! included in T2's view after executing I2".
//!
//! First, if `T2-2` saw the result of `T1-3`, then we want to enforce
//! `deref b @ T1 ⊑ free b @ T2` (recall the synchronization in `Arc::drop`). To enforce this, it
//! suffices to add release-acquire synchronization between `T1-3` and `T2-2`.
//!
//! For the second case, release-acquire doesn't guarantee "either `T1-1 ⊑ T2-2` or `T2-1 ⊑ T1-2`"
//! because `T1-2` may not read the message of `T2-1` and `T2-2` may not read the message of `T1-2`
//! at the same time, leading to concurrent `deref b` and `free b`. To make this work, we should
//! use SC fence (`fence(SeqCst)`). Recall that a SC fence joins the executing thread's view and
//! the global SC view. This means that the view of a thread after executing its SC fence is
//! entirely included in the view of another thread after its SC fence. If we insert a SC fence
//! between `T1-1` and `T1-2`, and another between `T2-1` and `T2-2`, then either
//! `T1's fence ⊑ T2's fence` or `T2's fence ⊑ T1's fence` holds. Therefore, `T1-1 ⊑ T2-2` or
//! `T2-1 ⊑ T1-2`.

use core::cell::RefCell;

#[cfg(feature = "check-loom")]
use loom::thread_local;
#[cfg(not(feature = "check-loom"))]
use std::thread_local;

mod hazard;
mod retire;

pub use hazard::{HazardBag, Shield};
pub use retire::RetiredSet;

#[cfg(not(feature = "check-loom"))]
/// Default global bag of all hazard pointers.
pub static HAZARDS: HazardBag = HazardBag::new();

#[cfg(feature = "check-loom")]
loom::lazy_static! {
    /// Default global bag of all hazard pointers.
    pub static ref HAZARDS: HazardBag = HazardBag::new();
}

thread_local! {
    /// Default thread-local retired pointer list.
    static RETIRED: RefCell<RetiredSet<'static>> = RefCell::new(RetiredSet::default());
}

/// Retires a pointer.
///
/// # Safety
///
/// * `pointer` must be removed from shared memory before calling this function.
/// * Subsumes the safety requirements of [`Box::from_raw`].
///
/// [`Box::from_raw`]: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw
pub unsafe fn retire<T>(pointer: *const T) {
    RETIRED.with(|r| r.borrow_mut().retire(pointer));
}

/// Frees the pointers that are `retire`d by the current thread and not `protect`ed by any other
/// threads.
pub fn collect() {
    RETIRED.with(|r| r.borrow_mut().collect());
}
