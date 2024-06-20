use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

/// Raw lock interface.
///
/// # Safety
///
/// Implementations of this trait must ensure that the lock is actually
/// exclusive: a lock can't be acquired while the lock is already locked.
// TODO: For weak memory, there needs to be a bit more stricter condition. unlock -hb→ lock.
pub unsafe trait RawLock: Default + Send + Sync {
    /// Raw lock's token type.
    type Token;

    /// Acquires the raw lock.
    fn lock(&self) -> Self::Token;

    /// Releases the raw lock.
    ///
    /// # Safety
    ///
    /// `unlock()` should be called with the token given by the corresponding `lock()`.
    unsafe fn unlock(&self, token: Self::Token);
}

/// Raw lock interface for the try_lock API.
///
/// # Safety
///
/// Implementations of this trait must ensure that the lock is actually
/// exclusive: a lock can't be acquired while the lock is already locked.
/// Also, `try_lock()`, when successful, should return a token that can be used for
/// `RawLock::unlock`.
pub unsafe trait RawTryLock: RawLock {
    /// Tries to acquire the raw lock.
    fn try_lock(&self) -> Result<Self::Token, ()>;
}

/// A type-safe lock.
#[repr(C)]
#[derive(Debug, Default)]
pub struct Lock<L: RawLock, T> {
    inner: L,
    data: UnsafeCell<T>,
}

unsafe impl<L: RawLock, T: Send> Send for Lock<L, T> {}
unsafe impl<L: RawLock, T: Send> Sync for Lock<L, T> {}

impl<L: RawLock, T> Lock<L, T> {
    /// Creates a new lock.
    pub fn new(data: T) -> Self {
        Self {
            inner: L::default(),
            data: UnsafeCell::new(data),
        }
    }

    /// Destroys the lock and retrieves the lock-protected value.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Acquires the lock and dereferences the inner value.
    pub fn lock(&self) -> LockGuard<L, T> {
        let token = self.inner.lock();
        LockGuard {
            lock: self,
            token: ManuallyDrop::new(token),
        }
    }
}

impl<L: RawTryLock, T> Lock<L, T> {
    /// Tries to acquire the lock and dereferences the inner value.
    pub fn try_lock(&self) -> Result<LockGuard<L, T>, ()> {
        self.inner.try_lock().map(|token| LockGuard {
            lock: self,
            token: ManuallyDrop::new(token),
        })
    }
}

/// A guard that holds the lock and dereferences the inner value.
#[derive(Debug)]
pub struct LockGuard<'s, L: RawLock, T> {
    lock: &'s Lock<L, T>,
    token: ManuallyDrop<L::Token>,
}

unsafe impl<L: RawLock, T: Send> Send for LockGuard<'_, L, T> {}
unsafe impl<L: RawLock, T: Sync> Sync for LockGuard<'_, L, T> {}

impl<L: RawLock, T> Drop for LockGuard<'_, L, T> {
    fn drop(&mut self) {
        // SAFETY: `self.token` is not used anymore in this function, and as we are `drop`ing
        // `self`, it is not used anymore.
        let token = unsafe { ManuallyDrop::take(&mut self.token) };

        // SAFETY: since `self` was created with `lock` and it's `token`, the `token` given to
        // `unlock()` is correct.
        unsafe { self.lock.inner.unlock(token) };
    }
}

impl<L: RawLock, T> Deref for LockGuard<'_, L, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Having a `LockGuard` means the underlying lock is acquired.
        unsafe { &*self.lock.data.get() }
    }
}

impl<L: RawLock, T> DerefMut for LockGuard<'_, L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Having a `LockGuard` means the underlying lock is held.
        unsafe { &mut *self.lock.data.get() }
    }
}

#[cfg(test)]
pub mod tests {
    use std::thread::scope;

    use super::{Lock, RawLock};

    pub fn smoke<L: RawLock>() {
        const LENGTH: usize = 1024;
        let d = Lock::<L, Vec<usize>>::default();

        scope(|s| {
            let d = &d;
            for i in 1..LENGTH {
                s.spawn(move || {
                    let mut d = d.lock();
                    d.push(i);
                });
            }
        });

        let mut d = d.into_inner();
        d.sort_unstable();
        assert_eq!(d, (1..LENGTH).collect::<Vec<usize>>());
    }
}
