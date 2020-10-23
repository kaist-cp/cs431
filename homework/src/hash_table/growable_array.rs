//! Growable array.

use core::marker::PhantomData;

/// Growable array.
#[derive(Debug)]
pub struct GrowableArray<V> {
    _marker: PhantomData<V>,
}
