//! Lock-free data structures.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

extern crate crossbeam_epoch;
extern crate crossbeam_utils;

#[macro_use]
mod utils;
pub mod list;
mod queue;
mod stack;

pub use list::List;
pub use queue::Queue;
pub use stack::Stack;
