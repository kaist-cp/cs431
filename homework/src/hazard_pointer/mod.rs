//! Hazard pointers.
//!
//! # Example
//!
//! ```
//! use std::sync::atomic::Ordering;
//! use cs492_concur_homework::hazard_pointer::{get_protected, retire, collect, Atomic, Shared};
//!
//! let atomic = Atomic::new(1);
//! let shield = get_protected(&atomic).unwrap();
//! assert_eq!(unsafe { *shield.deref() }, 1);
//!
//! // unlink the block and retire
//! atomic.store(Shared::null(), Ordering::Relaxed);
//! retire(shield.shared());
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
//! First, if `T1-3 → T2-2`, then `b` is freed after all accesses.
//!
//! Second, in all of the remaining cases, either `T1-1 → T2-2` or `T2-1 → T1-2` holds (otherwise,
//! there is a cycle).
//! - If `T1-1 → T2-2`, then `b` is not freed.
//! - If `T2-1 → T1-2`, then the validation fails, so `T1` will not deref `b`.
//!
//! Therefore the algorithm is safe in sequentially consistent memory model. However, this is not
//! true in relaxed memory model (construction of counterexamples is left as an exercise). To fix
//! this, we should synchronize the operations using ordering primitives.
//!
//! First, if `T2-2` saw the result of `T1-3`, then `deref b` by `T1` should happen before `free b`
//! by `T2`. To enforce this, it suffices to add release-acquire synchronization between `T1-3` and
//! `T2-2`.
//!
//! For the second case, release-acquire doesn't guarantee "either `T1-1 → T2-2` or `T2-1 → T1-2`"
//! because `T1-2` may not read the message of `T2-1` and `T2-2` may not read the message of `T1-2`
//! at the same time, leading to concurrent `deref b` and `free b`. To make this work, we should
//! use SC fence (`fence(SeqCst)`). Recall that SC fence joins the executing thread's view and the
//! global view. So there is a total order among all SC fences and a SC fence happens-before
//! another SC fence. If we insert a SC fence between `T1-1` and `T1-2`, and another between `T2-1`
//! and `T2-2`, then either `T1's fence → T2's fence` or `T2's fence → T1's fence` holds.
//! Therefore, `T1-1 → T2-2` or `T2-1 → T1-2`.

use core::cell::RefCell;
use std::thread;

#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{fence, Ordering};
#[cfg(feature = "check-loom")]
use loom::sync::atomic::{fence, Ordering};

#[cfg(feature = "check-loom")]
use loom::thread_local;
#[cfg(not(feature = "check-loom"))]
use std::thread_local;

mod align;
mod atomic;
mod hazard;
mod retire;

pub use atomic::{Atomic, Owned, Shared};
use hazard::Hazards;
pub use hazard::Shield;
use retire::Retirees;

#[cfg(not(feature = "check-loom"))]
/// Global set of all hazard pointers.
pub static HAZARDS: Hazards = Hazards::new();

#[cfg(feature = "check-loom")]
loom::lazy_static! {
    /// Global set of all hazard pointers.
    pub static ref HAZARDS: Hazards = Hazards::new();
}

thread_local! {
    /// Thread-local list of retired pointers. The first element of the pair is the machine
    /// representation of the pointer and the second is the function pointer to `free::<T>`.
    static RETIRED: RefCell<Retirees<'static>> = RefCell::new(Retirees::new(&HAZARDS));
}

/// Returns `None` if the current thread's hazard array is fully occupied. The returned shield must
/// be validated before using.
pub fn protect<T>(pointer: Shared<T>) -> Option<Shield<'static, T>> {
    todo!()
}

/// Returns a validated shield. Returns `None` if the current thread's hazard array is fully
/// occupied.
pub fn get_protected<T>(atomic: &Atomic<T>) -> Option<Shield<'static, T>> {
    todo!()
}

/// Retires a pointer.
pub fn retire<T>(pointer: Shared<T>) {
    RETIRED.with(|r| r.borrow_mut().retire(pointer));
}

/// Frees the pointers that are `retire`d by the current thread and not `protect`ed by any other
/// threads.
pub fn collect() {
    RETIRED.with(|r| r.borrow_mut().collect());
}
