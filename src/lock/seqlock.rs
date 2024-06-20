//! A sequence lock.

use core::mem;
use core::ops::Deref;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{fence, AtomicUsize};

use crossbeam_utils::Backoff;

/// A raw sequence lock.
#[derive(Debug)]
pub struct RawSeqLock {
    /// - Even: unlocked or read-locked.
    /// - Odd: write-locked.
    /// - Is monotonically increasing. In particuler, a large part of the API is unsafe to enforce
    ///   this.
    seq: AtomicUsize,
}

impl Default for RawSeqLock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawSeqLock {
    /// Creates a new raw sequence lock.
    pub const fn new() -> Self {
        Self {
            seq: AtomicUsize::new(0),
        }
    }

    /// Acquires a writer's lock.
    pub fn write_lock(&self) -> usize {
        let backoff = Backoff::new();

        loop {
            let seq = self.seq.load(Relaxed);
            if seq & 1 == 0
                && self
                    .seq
                    .compare_exchange(seq, seq.wrapping_add(1), Acquire, Relaxed)
                    .is_ok()
            {
                fence(Release);
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Releases a writer's lock.
    ///
    /// # Safety
    ///
    /// - `self` must be a an acquired writer's lock.
    /// - `seq` must be the be the value returned from the corresponding of the `write_lock()`.
    pub unsafe fn write_unlock(&self, seq: usize) {
        self.seq.store(seq.wrapping_add(2), Release);
    }

    /// Acquires a reader's lock.
    pub fn read_begin(&self) -> usize {
        let backoff = Backoff::new();

        loop {
            let seq = self.seq.load(Acquire);
            if seq & 1 == 0 {
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Validates reads.
    ///
    /// If `self` is a read lock and `seq` is the corresponding sequence number,
    /// then if the return value is `true`, the reads are valid.
    pub fn read_validate(&self, seq: usize) -> bool {
        fence(Acquire);

        seq == self.seq.load(Relaxed)
    }

    /// # Safety
    ///
    /// - `seq` must be even.
    pub unsafe fn upgrade(&self, seq: usize) -> bool {
        if self
            .seq
            .compare_exchange(seq, seq.wrapping_add(1), Acquire, Relaxed)
            .is_err()
        {
            return false;
        }

        fence(Release);
        true
    }
}

/// A sequence lock.
#[derive(Debug, Default)]
pub struct SeqLock<T> {
    inner: RawSeqLock,
    data: T,
}

/// A writer's lock guard.
#[derive(Debug)]
pub struct WriteGuard<'s, T> {
    lock: &'s SeqLock<T>,
    seq: usize,
}

/// A reader's lock guard.
#[derive(Debug)]
pub struct ReadGuard<'s, T> {
    lock: &'s SeqLock<T>,
    seq: usize,
}

// TODO: Think about the safety of these implementations.
unsafe impl<T: Send> Send for SeqLock<T> {}
unsafe impl<T: Send + Sync> Sync for SeqLock<T> {}

unsafe impl<T> Send for WriteGuard<'_, T> {}
unsafe impl<T: Sync> Sync for WriteGuard<'_, T> {}

unsafe impl<T> Send for ReadGuard<'_, T> {}
unsafe impl<T: Sync> Sync for ReadGuard<'_, T> {}

impl<T> SeqLock<T> {
    /// Creates a new sequence lock.
    pub const fn new(data: T) -> Self {
        SeqLock {
            inner: RawSeqLock::new(),
            data,
        }
    }

    /// Consumes this seqlock, returning the underlying data.
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Dereferences the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Acquires a writer's lock.
    pub fn write_lock(&self) -> WriteGuard<T> {
        let seq = self.inner.write_lock();
        WriteGuard { lock: self, seq }
    }

    /// # Safety
    ///
    /// All reads from the underlying data should be atomic.
    pub unsafe fn read_lock(&self) -> ReadGuard<T> {
        let seq = self.inner.read_begin();
        ReadGuard { lock: self, seq }
    }

    /// # Safety
    ///
    /// All reads from the underlying data should be atomic.
    pub unsafe fn read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = unsafe { self.read_lock() };
        let result = f(&guard);

        if guard.finish() {
            Some(result)
        } else {
            None
        }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY:
        //
        // - A `WriteGuard` implies `self.lock.inner` is an acquired write lock.
        // - `self.seq` is the proper sequence number of the write lock.
        unsafe { self.lock.inner.write_unlock(self.seq) };
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Clone for ReadGuard<'_, T> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock,
            seq: self.seq,
        }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        // HACK(@jeehoonkang): we really need linear type here:
        // https://github.com/rust-lang/rfcs/issues/814
        panic!("`seqlock::ReadGuard` should never drop. Use `ReadGuard::finish()` instead.");
    }
}

impl<'s, T> ReadGuard<'s, T> {
    /// Validates reads.
    pub fn validate(&self) -> bool {
        self.lock.inner.read_validate(self.seq)
    }

    /// Restarts the read critical section.
    pub fn restart(&mut self) {
        self.seq = self.lock.inner.read_begin();
    }

    /// Releases the reader's lock.
    pub fn finish(self) -> bool {
        let result = self.validate();
        mem::forget(self);
        result
    }

    /// Tries to upgrade to a writer's lock.
    pub fn upgrade(self) -> Result<WriteGuard<'s, T>, ()> {
        // SAFETY:
        //
        // - `self.seq` is the proper sequence number of the read lock, hence even.
        let result = if unsafe { self.lock.inner.upgrade(self.seq) } {
            Ok(WriteGuard {
                lock: self.lock,
                seq: self.seq,
            })
        } else {
            Err(())
        };
        mem::forget(self);
        result
    }
}
