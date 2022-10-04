//! A sequence lock.

use core::mem;
use core::ops::Deref;
use core::sync::atomic::{fence, AtomicUsize, Ordering};

use crossbeam_utils::Backoff;

/// A raw sequence lock.
#[derive(Debug)]
pub struct RawSeqLock {
    seq: AtomicUsize,
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
            let seq = self.seq.load(Ordering::Relaxed);
            if seq & 1 == 0
                && self
                    .seq
                    .compare_exchange(
                        seq,
                        seq.wrapping_add(1),
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            {
                fence(Ordering::Release);
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Releases a writer's lock.
    pub fn write_unlock(&self, seq: usize) {
        self.seq.store(seq.wrapping_add(2), Ordering::Release);
    }

    /// Acquires a reader's lock.
    pub fn read_begin(&self) -> usize {
        let backoff = Backoff::new();

        loop {
            let seq = self.seq.load(Ordering::Acquire);
            if seq & 1 == 0 {
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Releases a reader's lock and validates the read.
    pub fn read_validate(&self, seq: usize) -> bool {
        fence(Ordering::Acquire);

        seq == self.seq.load(Ordering::Relaxed)
    }

    /// # Safety
    ///
    /// `seq` must be even.
    pub unsafe fn upgrade(&self, seq: usize) -> Result<(), ()> {
        if self
            .seq
            .compare_exchange(
                seq,
                seq.wrapping_add(1),
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(());
        }

        fence(Ordering::Release);
        Ok(())
    }
}

/// A sequence lock.
#[derive(Debug)]
pub struct SeqLock<T> {
    lock: RawSeqLock,
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

unsafe impl<T: Send> Send for SeqLock<T> {}
unsafe impl<T: Send> Sync for SeqLock<T> {}

unsafe impl<'s, T> Send for WriteGuard<'s, T> {}
unsafe impl<'s, T: Send + Sync> Sync for WriteGuard<'s, T> {}

unsafe impl<'s, T> Send for ReadGuard<'s, T> {}
unsafe impl<'s, T: Send + Sync> Sync for ReadGuard<'s, T> {}

impl<T> SeqLock<T> {
    /// Creates a new sequence lock.
    pub const fn new(data: T) -> Self {
        SeqLock {
            lock: RawSeqLock::new(),
            data,
        }
    }

    /// Dereferences the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Acquires a writer's lock.
    pub fn write_lock(&self) -> WriteGuard<T> {
        let seq = self.lock.write_lock();
        WriteGuard { lock: self, seq }
    }

    /// # Safety
    ///
    /// All reads from the underlying data should be atomic.
    pub unsafe fn read_lock(&self) -> ReadGuard<T> {
        let seq = self.lock.read_begin();
        ReadGuard { lock: self, seq }
    }

    /// # Safety
    ///
    /// All reads from the underlying data should be atomic.
    pub unsafe fn read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.read_lock();
        let result = f(&guard);

        if guard.finish() {
            Some(result)
        } else {
            None
        }
    }
}

impl<'s, T> Deref for WriteGuard<'s, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<'s, T> Drop for WriteGuard<'s, T> {
    fn drop(&mut self) {
        self.lock.lock.write_unlock(self.seq);
    }
}

impl<'s, T> Deref for ReadGuard<'s, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<'s, T> Clone for ReadGuard<'s, T> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock,
            seq: self.seq,
        }
    }
}

impl<'s, T> Drop for ReadGuard<'s, T> {
    fn drop(&mut self) {
        // HACK(@jeehoonkang): we really need linear type here:
        // https://github.com/rust-lang/rfcs/issues/814
        panic!("seqlock::ReadGuard should never drop: use Self::finish() instead");
    }
}

impl<'s, T> ReadGuard<'s, T> {
    /// Validates the read.
    pub fn validate(&self) -> bool {
        self.lock.lock.read_validate(self.seq)
    }

    /// Restarts the read critical section.
    pub fn restart(&mut self) {
        self.seq = self.lock.lock.read_begin();
    }

    /// Releases the reader's lock.
    pub fn finish(self) -> bool {
        let result = self.lock.lock.read_validate(self.seq);
        mem::forget(self);
        result
    }

    /// Tries to upgrade to a writer's lock.
    pub fn upgrade(self) -> Result<WriteGuard<'s, T>, ()> {
        let result = if unsafe { self.lock.lock.upgrade(self.seq) }.is_ok() {
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
