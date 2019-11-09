use core::mem;
use core::ops::Deref;
use core::sync::atomic::{fence, AtomicUsize, Ordering};

use crossbeam_utils::Backoff;

#[derive(Debug)]
pub struct RawSeqLock {
    seq: AtomicUsize,
}

impl RawSeqLock {
    pub const fn new() -> Self {
        Self {
            seq: AtomicUsize::new(0),
        }
    }

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

    pub fn write_unlock(&self, seq: usize) {
        self.seq.store(seq.wrapping_add(2), Ordering::Release);
    }

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

    pub fn read_validate(&self, seq: usize) -> bool {
        fence(Ordering::Acquire);

        seq == self.seq.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct SeqLock<T> {
    lock: RawSeqLock,
    data: T,
}

#[derive(Debug)]
pub struct WriteGuard<'s, T> {
    lock: &'s SeqLock<T>,
    seq: usize,
}

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
    pub const fn new(data: T) -> Self {
        SeqLock {
            lock: RawSeqLock::new(),
            data,
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

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

impl<'s, T> ReadGuard<'s, T> {
    pub fn validate(&self) -> bool {
        self.lock.lock.read_validate(self.seq)
    }

    pub fn restart(&mut self) {
        self.seq = self.lock.lock.read_begin();
    }

    pub fn finish(self) -> bool {
        let result = self.lock.lock.read_validate(self.seq);
        mem::forget(self);
        result
    }
}

impl<'s, T> Drop for ReadGuard<'s, T> {
    fn drop(&mut self) {
        // HACK(@jeehoonkang): we really need linear type here:
        // https://github.com/rust-lang/rfcs/issues/814
        panic!("seqlock::ReadGuard should never drop: use Self::finish() instead");
    }
}
