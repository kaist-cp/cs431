use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;
use crossbeam_epoch::{Guard, Owned, Shared};
use std::thread;

use super::base::{Stack, ELIM_DELAY, ElimStack};

impl<T, S: Stack<T>> Stack<T> for ElimStack<T, S> {
    type PushReq = S::PushReq;

    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>> {
        let req = match self.inner.try_push(req, guard) {
            Ok(()) => return Ok(()),
            Err(req) => req,
        };

        unimplemented!()
    }

    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        if let Ok(result) = self.inner.try_pop(guard) {
            return Ok(result);
        }

        unimplemented!()
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.inner.is_empty(guard)
    }
}
