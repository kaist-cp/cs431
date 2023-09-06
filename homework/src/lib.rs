//! Homeworks

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![allow(clippy::result_unit_err)]
// Allow lints for homework.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

mod adt;
mod arc;
mod elim_stack;
mod hash_table;
pub mod hazard_pointer;
pub mod hello_server;
mod linked_list;
mod list_set;
mod set;

pub mod test;

pub use adt::{
    ConcurrentMap, ConcurrentSet, NonblockingConcurrentMap, NonblockingMap, SequentialMap,
};
pub use arc::Arc;
pub use elim_stack::ElimStack;
pub use hash_table::{GrowableArray, SplitOrderedList};
pub use linked_list::LinkedList;
pub use list_set::{
    fine_grained::FineGrainedListSet, optimistic_fine_grained::OptimisticFineGrainedListSet,
};
