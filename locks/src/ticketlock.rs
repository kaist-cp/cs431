use core::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_utils::Backoff;

pub struct TicketLock {
    curr: AtomicUsize,
    next: AtomicUsize,
}

impl TicketLock {
    pub const fn new() -> Self {
        Self {
            curr: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }

    pub fn lock(&self) -> usize {
        let ticket = self.next.fetch_add(1, Ordering::Relaxed);
        let backoff = Backoff::new();

        while self.curr.load(Ordering::Acquire) != ticket {
            backoff.snooze();
        }

        ticket
    }

    pub fn unlock(&self, ticket: usize) {
        self.curr.store(ticket.wrapping_add(1), Ordering::Release);
    }
}
