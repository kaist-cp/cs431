use core::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_utils::Backoff;

use crate::lock::*;

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

impl RawLock for TicketLock {
    type Token = usize;

    fn lock(&self) -> usize {
        let ticket = self.next.fetch_add(1, Ordering::Relaxed);
        let backoff = Backoff::new();

        while self.curr.load(Ordering::Acquire) != ticket {
            backoff.snooze();
        }

        ticket
    }

    unsafe fn unlock(&self, ticket: usize) {
        self.curr.store(ticket.wrapping_add(1), Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use crate::ticketlock::TicketLock;

    #[test]
    fn smoke() {
        crate::lock::tests::smoke::<TicketLock>();
    }
}
