use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;
use std::thread;

use crossbeam_epoch::{Guard, Owned, Shared};

use super::base::{get_random_elim_index, ElimStack, Stack, ELIM_DELAY};

impl<T, S: Stack<T>> Stack<T> for ElimStack<T, S> {
    type PushReq = S::PushReq;

    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>> {
        let Err(req) = self.inner.try_push(req, guard) else {
            return Ok(());
        };

        let index = get_random_elim_index();
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        todo!()
    }

    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        if let Ok(result) = self.inner.try_pop(guard) {
            return Ok(result);
        }

        let index = get_random_elim_index();
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        todo!()
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.inner.is_empty(guard)
    }
}
