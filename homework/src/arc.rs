//! Simplified `Arc`.
//!
//! See the `Arc` documentation for more details and specification.

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

/// Simplified `Arc` without `Weak` support.
///
/// The main correctness guarantee of `Arc` is that the deallocation of its data and counter field
/// happens-after all accesses to those fields.  An access (by `Deref::deref`, `get_mut`, ...) to an
/// `Arc` happen between the construction of that `Arc` (by `new` or `clone`) and its destruction (by
/// `drop`).  The fields are deallocated when the last `Arc` pointing to the same fields is
/// `drop`ped.  Therefore, the correctness guarantee translates to making sure that the
/// deallocation done by the last `drop` happens-after all previous `drop`'s.
///
/// `get_mut` and `make_mut`, the methods that obtain a temporary exclusive reference (`&mut`) to
/// the underlying data, provide an additional guarantee that the returned reference is indeed
/// exclusive.  In the concurrent setting, this means that obtaining the exclusive reference
/// happens-after all the other accesses to the content. Since all those accesses can only happen
/// between the construction and the destruction of an `Arc`, it follows that the creation of the
/// exclusive reference happens-after `drop`s of all the other `Arc`s pointing to the same content.
/// `try_unwrap` also provides a similar guarantee as it returns the exclusive ownership of the
/// data.
///
/// The above explanation is based on the paper [RustBelt Meets Relaxed Memory by Dang et
/// al.](https://plv.mpi-sws.org/rustbelt/rbrlx/).
pub struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    fn from_inner(ptr: NonNull<ArcInner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

struct ArcInner<T> {
    count: AtomicUsize,
    data: T,
}

unsafe impl<T: Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: Sync + Send> Sync for ArcInner<T> {}

impl<T> Arc<T> {
    /// Constructs a new `Arc<T>`.
    #[inline]
    pub fn new(data: T) -> Arc<T> {
        let x = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data,
        });
        Self::from_inner(Box::leak(x).into())
    }

    /// Returns a mutable reference into the given `Arc` if there are
    /// no other `Arc`. Otherwise, return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let mut x = Arc::new(3);
    /// *Arc::get_mut(&mut x).unwrap() = 4;
    /// assert_eq!(*x, 4);
    ///
    /// let y = Arc::clone(&x);
    /// assert!(Arc::get_mut(&mut x).is_none());
    ///
    /// drop(y);
    /// assert!(Arc::get_mut(&mut x).is_some());
    /// ```
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        todo!()
    }

    // Used in `get_mut` and `make_mut` to check if the given `Arc` is the unique reference to the
    // underlying data.
    #[inline]
    fn is_unique(&mut self) -> bool {
        todo!()
    }

    /// Returns a mutable reference into the given `Arc` without any check.
    ///
    /// # Safety
    ///
    /// Any other `Arc` to the same allocation must not be dereferenced for the duration of the
    /// returned borrow.  Specifically, call to this function must happen-after destruction of all
    /// the other `Arc` to the same allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let mut x = Arc::new(String::new());
    /// unsafe {
    ///     Arc::get_mut_unchecked(&mut x).push_str("foo")
    /// }
    /// assert_eq!(*x, "foo");
    /// ```
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts.
        &mut (*this.ptr.as_ptr()).data
    }

    /// Gets the number of `Arc`s to this allocation. In addition, synchronize with the update that
    /// this function reads from.
    ///
    /// # Safety
    ///
    /// This method by itself is safe, but using it correctly requires extra care.
    /// Another thread can change the reference count at any time,
    /// including potentially between calling this method and acting on the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let five = Arc::new(5);
    /// let _also_five = Arc::clone(&five);
    ///
    /// // This assertion is deterministic because we haven't shared
    /// // the `Arc` between threads.
    /// assert_eq!(2, Arc::count(&five));
    /// ```
    #[inline]
    pub fn count(this: &Self) -> usize {
        todo!()
    }

    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }

    /// Returns `true` if the two `Arc`s point to the same allocation
    /// (in a vein similar to `ptr::eq`).
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let five = Arc::new(5);
    /// let same_five = Arc::clone(&five);
    /// let other_five = Arc::new(5);
    ///
    /// assert!(Arc::ptr_eq(&five, &same_five));
    /// assert!(!Arc::ptr_eq(&five, &other_five));
    /// ```
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }

    /// Returns the inner value, if the given `Arc` is unique.
    ///
    /// Otherwise, an `Err` is returned with the same `Arc` that was passed in.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let x = Arc::new(3);
    /// assert_eq!(Arc::try_unwrap(x).unwrap(), 3);
    ///
    /// let x = Arc::new(4);
    /// let _y = Arc::clone(&x);
    /// assert_eq!(*Arc::try_unwrap(x).unwrap_err(), 4);
    /// ```
    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        todo!()
    }
}

impl<T: Clone> Arc<T> {
    /// Makes a mutable reference into the given `Arc`.
    ///
    /// If there are other `Arc` to the same allocation, then `make_mut` will create a new
    /// allocation and invoke `clone` on the inner value to ensure unique ownership. This is also
    /// referred to as clone-on-write.
    ///
    /// See also `get_mut`, which will fail rather than cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let mut data = Arc::new(5);
    ///
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// let mut other_data = Arc::clone(&data); // Won't clone inner data
    /// *Arc::make_mut(&mut data) += 1;         // Clones inner data
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// *Arc::make_mut(&mut other_data) *= 2;   // Won't clone anything
    ///
    /// // Now `data` and `other_data` point to different allocations.
    /// assert_eq!(*data, 8);
    /// assert_eq!(*other_data, 12);
    /// ```
    #[inline]
    pub fn make_mut(this: &mut Self) -> &mut T {
        todo!()
    }
}

impl<T> Clone for Arc<T> {
    /// Makes a clone of the `Arc` pointer.
    ///
    /// This creates another pointer to the same allocation, increasing the
    /// reference count.
    ///
    /// # Panics
    ///
    /// This panics if the number of `Arc`s is larger than `isize::Max`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// let _ = Arc::clone(&five);
    /// ```
    #[inline]
    fn clone(&self) -> Arc<T> {
        todo!()
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T> Drop for Arc<T> {
    /// Drops the `Arc`.
    ///
    /// This will decrement the reference count. If the reference
    /// count reaches zero, we `drop` the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cs492_concur_homework::Arc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Arc::new(Foo);
    /// let foo2 = Arc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    fn drop(&mut self) {
        todo!()
    }
}

impl<T: fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}
