//! Locks.

mod api;
mod clhlock;
mod mcslock;
mod mcsparkinglock;
pub mod seqlock;
mod spinlock;
mod ticketlock;

pub use api::{Lock, LockGuard, RawLock, RawTryLock};
pub use clhlock::ClhLock;
pub use mcslock::McsLock;
pub use mcsparkinglock::McsParkingLock;
pub use spinlock::SpinLock;
pub use ticketlock::TicketLock;
