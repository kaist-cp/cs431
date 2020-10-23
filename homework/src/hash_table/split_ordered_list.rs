//! Split-ordered linked list.

use core::marker::PhantomData;

/// Split-ordered list.
#[derive(Debug)]
pub struct SplitOrderedList<V> {
    _marker: PhantomData<V>,
}
