use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use crossbeam_utils::Backoff;

use crate::lock::*;

/// A ticket lock.
#[derive(Debug)]
pub struct TicketLock {
    curr: AtomicUsize,
    next: AtomicUsize,
}

impl Default for TicketLock {
    fn default() -> Self {
        Self {
            curr: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }
}

unsafe impl RawLock for TicketLock {
    type Token = usize;

    fn lock(&self) -> usize {
        let ticket = self.next.fetch_add(1, Relaxed);
        let backoff = Backoff::new();

        while self.curr.load(Acquire) != ticket {
            backoff.snooze();
        }

        ticket
    }

    unsafe fn unlock(&self, ticket: usize) {
        self.curr.store(ticket.wrapping_add(1), Release);
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::ticketlock::TicketLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<TicketLock>();
    }
}
