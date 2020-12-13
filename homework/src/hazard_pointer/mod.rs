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
//! (T1-1) add b to the hazard list                   | (T2-1) unlink b (and retire b)
//! (T1-2) check if b is still reachable (validation) | (T2-2) check if b is in the hazard list
//!        if so, deref b                             |        if not, free b
//! ```
//!
//! In the sequentially consistent memory model, there are 6 interleavings of each step.
//! In any of them, either `T1-1 → T2-2` or `T2-1 → T1-2` holds.
//! - If `T1-1 → T2-2`, then b is not freed, so no use-after-**free**
//! - If `T2-1 → T1-2`, then the validation fails, so T1 will not read. No **use**-after-free.
//!
//! Therefore the algorithm is safe in sequentially consistent memory model. However, this is not
//! true in relaxed memory model (construction of a counterexample is left as an exercise).
//!
//! To make the above reasoning sound in the relaxed memory model, we should use SC fence
//! (`fence(SeqCst)`). Recall that SC fence joins the executing thread's view and the global view.
//! So there is a total order among all SC fences and a SC fence happens-before another SC fence.
//! If we insert a SC fence between `T1-1` and `T1-2`, and another between `T2-1` and `T2-2`, then
//! either `T1's fence → T2's fence` or `T2's fence → T1's fence` holds.
//! Therefore, `T1-1 → T2-2` or `T2-1 → T1-2`.
use core::cell::RefCell;
use std::sync::atomic::{fence, Ordering};
use std::thread;

mod align;
mod atomic;
mod hazard;
mod retire;

pub use atomic::{Atomic, Owned, Shared};
use hazard::Hazards;
pub use hazard::Shield;
use retire::Retirees;

/// Global set of all hazard pointers.
static HAZARDS: Hazards = Hazards::new();

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
