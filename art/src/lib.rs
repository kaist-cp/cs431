//! Adaptive radix tree.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate itertools;
#[macro_use]
extern crate static_assertions;
extern crate crossbeam_epoch;
extern crate crossbeam_utils;
extern crate either;
extern crate lock;
extern crate rand;

#[macro_use]
mod utils;
mod art;
mod map;
mod node;

pub use art::{Art, Entry};
pub use map::{ConcurrentMap, SequentialMap};
