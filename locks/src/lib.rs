#![feature(const_fn)]

mod clhlock;
mod lock;
mod spinlock;
mod ticketlock;

pub use clhlock::ClhLock;
pub use lock::{Lock, LockGuard, RawLock, RawTryLock};
pub use spinlock::SpinLock;
pub use ticketlock::TicketLock;
