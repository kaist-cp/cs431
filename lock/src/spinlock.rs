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

        while self
            .inner
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            backoff.snooze();
        }
    }

    unsafe fn unlock(&self, _token: ()) {
        self.inner.store(false, Ordering::Release);
    }
}

impl RawTryLock for SpinLock {
    fn try_lock(&self) -> Result<(), ()> {
        self.inner
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| ())
            .map_err(|_| ())
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
