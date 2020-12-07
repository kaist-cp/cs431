//! Hazard pointers.
//!
//! # Example
//!
//! ```
//! use std::sync::atomic::Ordering;
//! use cs492_concur_homework::hazard_pointer::{protect, retire, collect, Atomic, Shared};
//!
//! let atomic = Atomic::new(1);
//! let mut shared = atomic.load(Ordering::Acquire);
//! let shield = loop {
//!     let shield = protect(shared).unwrap();
//!     let shared2 = atomic.load(Ordering::Relaxed);
//!     // validate if `atomic` still points to the pointer that I protected.
//!     if shield.validate(shared2) {
//!         break shield;
//!     }
//!     shared = shared2;
//! };
//! // protection of `shared` is established
//! assert_eq!(unsafe { *shield.deref() }, 1);
//!
//! // unlink the block and retire
//! atomic.store(Shared::null(), Ordering::Relaxed);
//! retire(shield.shared());
//!
//! // manually trigger reclamation
//! collect();
//! ```
//!
use core::cell::RefCell;
use lazy_static::lazy_static;
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

lazy_static! {
    /// Global set of all hazard pointers.
    static ref HAZARDS: Hazards = Hazards::new();
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

/// Retires a pointer.
pub fn retire<T>(pointer: Shared<T>) {
    RETIRED.with(|r| r.borrow_mut().retire(pointer));
}

/// Frees the pointers that are `retire`d by the current thread and not `protect`ed by any other
/// threads.
pub fn collect() {
    RETIRED.with(|r| r.borrow_mut().collect());
}
