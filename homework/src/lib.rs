//! Homeworks

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![allow(clippy::result_unit_err)]
// Allow lints for homework.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

#[macro_use]
mod utils;

mod arc;
mod art;
mod bst;
mod elim_stack;
mod hash_table;
pub mod hazard_pointer;
pub mod hello_server;
mod linked_list;
mod list_set;
mod map;

pub use arc::Arc;
pub use art::{Art, Entry};
pub use bst::Bst;
pub use elim_stack::ElimStack;
pub use hash_table::{GrowableArray, SplitOrderedList};
pub use linked_list::LinkedList;
pub use list_set::OrderedListSet;
pub use map::{
    ConcurrentMap, NonblockingConcurrentMap, NonblockingMap, RandGen, SequentialMap, StrStringMap,
};
