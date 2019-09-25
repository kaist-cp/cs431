#![feature(const_fn)]

extern crate crossbeam_utils;

mod clhlock;
mod lock;
mod mcslock;
mod mcsparkinglock;
mod spinlock;
mod ticketlock;

pub use clhlock::ClhLock;
pub use lock::{Lock, LockGuard, RawLock, RawTryLock};
pub use mcsparkinglock::McsParkingLock;
pub use spinlock::SpinLock;
pub use ticketlock::TicketLock;
