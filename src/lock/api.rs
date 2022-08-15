use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};

/// Raw lock interface.
pub trait RawLock: Default + Send + Sync {
    /// Raw lock's token type.
    type Token: Clone;

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
pub trait RawTryLock: RawLock {
    /// Tries to acquire the raw lock.
    fn try_lock(&self) -> Result<Self::Token, ()>;
}

/// A type-safe lock.
#[repr(C)]
#[derive(Debug)]
pub struct Lock<L: RawLock, T> {
    lock: L,
    data: UnsafeCell<T>,
}

unsafe impl<L: RawLock, T: Send> Send for Lock<L, T> {}
unsafe impl<L: RawLock, T: Send> Sync for Lock<L, T> {}

impl<L: RawLock, T> Lock<L, T> {
    /// Creates a new lock.
    pub fn new(data: T) -> Self {
        Self {
            lock: L::default(),
            data: UnsafeCell::new(data),
        }
    }

    /// Destroys the lock and retrieves the lock-protected value.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Acquires the lock and dereferences the inner value.
    pub fn lock(&self) -> LockGuard<L, T> {
        let token = self.lock.lock();
        LockGuard {
            lock: self,
            token,
            _marker: PhantomData,
        }
    }
}

impl<L: RawTryLock, T> Lock<L, T> {
    /// Tries to acquire the lock and dereferences the inner value.
    pub fn try_lock(&self) -> Result<LockGuard<L, T>, ()> {
        self.lock.try_lock().map(|token| LockGuard {
            lock: self,
            token,
            _marker: PhantomData,
        })
    }
}

impl<L: RawLock, T> Lock<L, T> {
    /// # Safety
    ///
    /// The underlying lock should be actually acquired.
    pub unsafe fn unlock_unchecked(&self, token: L::Token) {
        self.lock.unlock(token);
    }

    /// # Safety
    ///
    /// The underlying lock should be actually acquired.
    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.data.get()
    }

    /// Dereferences the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// # Safety
    ///
    /// The underlying lock should be actually acquired.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_unchecked(&self) -> &mut T {
        &mut *self.data.get()
    }
}

/// A guard that holds the lock and dereferences the inner value.
#[derive(Debug)]
pub struct LockGuard<'s, L: RawLock, T> {
    lock: &'s Lock<L, T>,
    token: L::Token,
    _marker: PhantomData<*const ()>, // !Send + !Sync
}

unsafe impl<'s, L: RawLock, T: Send> Send for LockGuard<'s, L, T> {}
unsafe impl<'s, L: RawLock, T: Sync> Sync for LockGuard<'s, L, T> {}

impl<'s, L: RawLock, T> LockGuard<'s, L, T> {
    /// Returns the address of the referenced lock.
    pub fn raw(&mut self) -> usize {
        self.lock as *const _ as usize
    }
}

impl<'s, L: RawLock, T> Drop for LockGuard<'s, L, T> {
    fn drop(&mut self) {
        unsafe { self.lock.lock.unlock(self.token.clone()) };
    }
}

impl<'s, L: RawLock, T> Deref for LockGuard<'s, L, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'s, L: RawLock, T> DerefMut for LockGuard<'s, L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'s, L: RawLock, T> LockGuard<'s, L, T> {
    /// Transforms a lock guard to an address.
    pub fn into_raw(self) -> usize {
        let ret = self.lock as *const _ as usize;
        mem::forget(self);
        ret
    }

    /// # Safety
    ///
    /// The given arguments should be the data of a forgotten lock guard.
    pub unsafe fn from_raw(data: usize, token: L::Token) -> Self {
        Self {
            lock: &*(data as *const _),
            token,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use core::ops::Deref;

    use crossbeam_utils::thread::scope;

    use super::{Lock, RawLock};

    pub fn smoke<L: RawLock>() {
        const LENGTH: usize = 1024;
        let d = Lock::<L, Vec<usize>>::new(vec![]);

        scope(|s| {
            for i in 1..LENGTH {
                let d = &d;
                s.spawn(move |_| {
                    let mut d = d.lock();
                    d.push(i);
                });
            }
        })
        .unwrap();

        let mut d = d.lock();
        d.sort();
        assert_eq!(d.deref(), &(1..LENGTH).collect::<Vec<usize>>());
    }
}
