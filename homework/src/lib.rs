//! Homeworks

#![warn(missing_docs, missing_debug_implementations, unreachable_pub)]
#![allow(clippy::result_unit_err)]
// Allow lints for homework.
#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
#![deny(unsafe_op_in_unsafe_fn, warnings)]

mod adt;
mod arc;
pub mod boc;
mod elim_stack;
mod hash_table;
pub mod hazard_pointer;
pub mod hello_server;
mod linked_list;
mod list_set;

pub mod test;

pub use adt::{ConcurrentMap, ConcurrentSet};
pub use arc::Arc;
pub use boc::CownPtr;
pub use elim_stack::ElimStack;
pub use hash_table::{GrowableArray, SplitOrderedList};
pub use linked_list::LinkedList;
pub use list_set::{FineGrainedListSet, OptimisticFineGrainedListSet};
