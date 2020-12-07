use core::sync::atomic::{fence, Ordering};

use super::align;
use super::atomic::Shared;
use super::hazard::Hazards;

/// Thread-local list of retired pointers.
pub struct Retirees<'s> {
    hazards: &'s Hazards,
    /// The first element of the pair is the machine representation of a pointer without tag and
    /// the second is the function pointer to `free::<T>` where `T` is the type of the object. 
    inner: Vec<(usize, unsafe fn(usize))>,
}

impl<'s> Retirees<'s> {
    /// The max length of retired pointer list. Call `collect` if the length becomes larger than
    /// this value.
    const THRESHOLD: usize = 64;

    pub fn new(hazards: &'s Hazards) -> Self {
        Self {
            hazards,
            inner: Vec::new(),
        }
    }

    /// Retire a pointer.
    pub fn retire<T>(&mut self, pointer: Shared<T>) {
        unsafe fn free<T>(data: usize) {
            debug_assert_eq!(align::decompose_tag::<T>(data).1, 0);
            drop(Box::from_raw(data as *mut T))
        }

        todo!()
    }

    /// Free the pointers that are `retire`d by the current thread and not `protect`ed by any other
    /// threads.
    pub fn collect(&mut self) {
        todo!()
    }
}

impl Drop for Retirees<'_> {
    fn drop(&mut self) {
        // In a production-quality implementation of hazard pointers, the remaining local retired
        // pointers will be moved to a global list of retired pointers, which are then reclaimed by
        // the other threads. For pedagogical purposes, here we simply wait for all retired pointers
        // are no longer protected.
        while !self.inner.is_empty() {
            self.collect();
        }
    }
}
