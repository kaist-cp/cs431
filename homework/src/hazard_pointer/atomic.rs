use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};

use super::align;

/// An owned heap-allocated object.
///
/// This type is very similar to `Box<T>`.
///
/// The pointer must be properly aligned. Since it is aligned, a tag can be stored into the unused
/// least significant bits of the address.
#[derive(Debug)]
pub struct Owned<T> {
    data: usize,
    _marker: PhantomData<Box<T>>,
}

/// An atomic pointer that can be safely shared between threads.
///
/// The pointer must be properly aligned. Since it is aligned, a tag can be stored into the unused
/// least significant bits of the address. For example, the tag for a pointer to a sized type `T`
/// should be less than `(1 << mem::align_of::<T>().trailing_zeros())`.
#[derive(Debug)]
pub struct Atomic<T> {
    data: AtomicUsize,
    _marker: PhantomData<*const T>,
}

/// A pointer to an object that can be protected by a `Shield`.
///
/// The pointer must be properly aligned. Since it is aligned, a tag can be stored into the unused
/// least significant bits of the address.
#[derive(Debug)]
pub struct Shared<T> {
    data: usize,
    _marker: PhantomData<*const T>,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Shared<T> {}

impl<T> Owned<T> {
    /// Allocates `data` on the heap and returns a new owned pointer pointing to it.
    pub fn new(data: T) -> Self {
        Self {
            data: Box::into_raw(Box::new(data)) as usize,
            _marker: PhantomData,
        }
    }

    /// Returns the tag stored within the pointer.
    pub fn tag(&self) -> usize {
        let (_, tag) = align::decompose_tag::<T>(self.data);
        tag
    }

    /// Returns the same pointer, but tagged with `tag`. `tag` is truncated to be fit into the
    /// unused bits of the pointer to `T`.
    pub fn with_tag(self, tag: usize) -> Self {
        let (data, _) = align::decompose_tag::<T>(self.data);
        Self {
            data: align::compose_tag::<T>(data, tag),
            _marker: PhantomData,
        }
    }

    /// Converts the owned pointer into a [`Shared`].
    pub fn into_shared(self) -> Shared<T> {
        let data = self.data;
        mem::forget(self);
        Shared::<T>::from_usize(data)
    }

    /// Returns a new pointer pointing to the tagged pointer `data`.
    ///
    /// # Panics
    ///
    /// Panics if the data is zero in debug mode.
    #[inline]
    unsafe fn from_usize(data: usize) -> Self {
        debug_assert!(data != 0, "converting zero into `Owned`");
        Owned {
            data,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Owned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data as *const T) }
    }
}

impl<T> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data as *mut T) }
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        let (data, _) = align::decompose_tag::<T>(self.data);
        drop(unsafe { Box::<T>::from_raw(data as *mut T) });
    }
}

unsafe impl<T: Send + Sync> Send for Atomic<T> {}
unsafe impl<T: Send + Sync> Sync for Atomic<T> {}

impl<T> Atomic<T> {
    /// Returns a new null atomic pointer.
    pub fn null() -> Self {
        Self {
            data: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Allocates `data` on the heap and returns a new atomic pointer pointing to it.
    pub fn new(data: T) -> Self {
        let data = AtomicUsize::new(Owned::new(data).into_shared().into_usize());
        Self {
            data,
            _marker: PhantomData,
        }
    }

    /// Loads a `Shared` from the atomic pointer.
    pub fn load(&self, ord: Ordering) -> Shared<T> {
        Shared::from_usize(self.data.load(ord))
    }

    /// Stores a `Shared` into the atomic pointer.
    pub fn store(&self, data: Shared<T>, ord: Ordering) {
        self.data.store(data.data, ord);
    }

    /// Stores the `Shared` pointer `new` into the atomic pointer if the current value is the same
    /// as `cur`. The tag is also taken into account, so two pointers to the same object, but with
    /// different tags, will not be considered equal.
    ///
    /// The return value is a result indicating whether the new pointer was written. On failure the
    /// actual current value is returned.
    pub fn compare_and_set(
        &self,
        cur: Shared<T>,
        new: Shared<T>,
        ord_succ: Ordering,
        ord_fail: Ordering,
    ) -> Result<(), Shared<T>> {
        self.data
            .compare_exchange(cur.data, new.data, ord_succ, ord_fail)
            .map(|_| ())
            .map_err(Shared::from_usize)
    }

    /// Performs a bitwise "or" operation on the current tag and the argument `tag`, and sets the
    /// new tag to the result. Returns the previous pointer.
    pub fn fetch_or(&self, tag: usize, ord: Ordering) -> Shared<T> {
        let tag = tag & align::low_bits::<T>();
        let old = self.data.fetch_or(tag, ord);
        Shared::from_usize(old)
    }
}

impl<T> Shared<T> {
    /// Returns a new null pointer.
    pub fn null() -> Shared<T> {
        Shared {
            data: 0,
            _marker: PhantomData,
        }
    }

    /// Returns the tag stored within the pointer.
    pub fn tag(&self) -> usize {
        let (_, tag) = align::decompose_tag::<T>(self.data);
        tag
    }

    /// Returns the same pointer, but tagged with `tag`. `tag` is truncated to be fit into the
    /// unused bits of the pointer to `T`.
    pub fn with_tag(self, tag: usize) -> Self {
        let (data, _) = align::decompose_tag::<T>(self.data);
        Self {
            data: align::compose_tag::<T>(data, tag),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the pointer is null ignoring its tag.
    pub fn is_null(&self) -> bool {
        let (data, _) = align::decompose_tag::<T>(self.data);
        data == 0
    }

    /// Returns the machine representation of the pointer.
    pub fn into_usize(self) -> usize {
        self.data
    }

    /// Returns a new pointer pointing to the tagged pointer `data`.
    pub fn from_usize(data: usize) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }

    /// Takes ownership of the pointee.
    ///
    /// # Panics
    ///
    /// Panics if this pointer is null, but only in debug mode.
    ///
    /// # Safety
    ///
    /// This method may be called only if the pointer is valid and nobody else is holding a
    /// reference to the same object.
    pub unsafe fn into_owned(self) -> Owned<T> {
        debug_assert!(!self.is_null(), "converting a null `Shared` into `Owned`");
        Owned::from_usize(self.data)
    }

    /// Dereferences the shared pointer.
    ///
    /// # Safety
    ///
    /// The pointer should be valid and the pointee should not be concurrently accessed by the
    /// other threads.
    pub unsafe fn deref(&self) -> &T {
        &*(self.data as *const T)
    }
}
