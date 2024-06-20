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

use core::cell::RefCell;
#[cfg(not(feature = "check-loom"))]
use std::thread_local;

#[cfg(feature = "check-loom")]
use loom::thread_local;

mod hazard;
mod retire;

pub use hazard::{HazardBag, Shield};
pub use retire::RetiredSet;

#[cfg(not(feature = "check-loom"))]
/// Default global bag of all hazard pointers.
pub static HAZARDS: HazardBag = HazardBag::new();

#[cfg(feature = "check-loom")]
// FIXME: loom does not currently provide the equivalent of Lazy:
// https://github.com/tokio-rs/loom/issues/263
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
/// * `pointer` must be removed from shared memory before calling this function, and must be valid.
/// * The same `pointer` should only be retired once.
pub unsafe fn retire<T>(pointer: *mut T) {
    RETIRED.with(|r| unsafe { r.borrow_mut().retire(pointer) });
}

/// Frees the pointers that are `retire`d by the current thread and not `protect`ed by any other
/// threads.
pub fn collect() {
    RETIRED.with(|r| r.borrow_mut().collect());
}
