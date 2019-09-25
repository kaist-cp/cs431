use core::sync::atomic::{AtomicBool, Ordering};

use crossbeam_utils::Backoff;

use crate::lock::*;

pub struct SpinLock {
    inner: AtomicBool,
}

impl Default for SpinLock {
    fn default() -> Self {
        Self {
            inner: AtomicBool::new(false),
        }
    }
}

impl RawLock for SpinLock {
    type Token = ();

    fn lock(&self) {
        let backoff = Backoff::new();

        while self.inner.compare_and_swap(false, true, Ordering::Acquire) {
            backoff.snooze();
        }
    }

    unsafe fn unlock(&self, _token: ()) {
        self.inner.store(false, Ordering::Release);
    }
}

impl RawTryLock for SpinLock {
    fn try_lock(&self) -> Result<(), ()> {
        if !self.inner.compare_and_swap(false, true, Ordering::Acquire) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spinlock::SpinLock;

    #[test]
    fn smoke() {
        crate::lock::tests::smoke::<SpinLock>();
    }
}
