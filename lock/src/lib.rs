#![allow(clippy::result_unit_err)]

extern crate crossbeam_utils;

mod clhlock;
mod lock;
mod mcslock;
mod mcsparkinglock;
pub mod seqlock;
mod spinlock;
mod ticketlock;

pub use crate::clhlock::ClhLock;
pub use crate::lock::{Lock, LockGuard, RawLock, RawTryLock};
pub use crate::mcslock::McsLock;
pub use crate::mcsparkinglock::McsParkingLock;
pub use crate::spinlock::SpinLock;
pub use crate::ticketlock::TicketLock;
