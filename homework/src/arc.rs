//! Thread-safe reference-counting pointers.
//!
//! See the [`Arc<T>`][Arc] documentation for more details.

use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
#[cfg(not(feature = "check-loom"))]
use std::sync::atomic::{fence, AtomicUsize, Ordering};
use std::{fmt, mem};

#[cfg(feature = "check-loom")]
use loom::sync::atomic::{fence, AtomicUsize, Ordering};

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

/// A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically
/// Reference Counted'.
///
/// The type `Arc<T>` provides shared ownership of a value of type `T`,
/// allocated in the heap. Invoking [`clone`][clone] on `Arc` produces
/// a new `Arc` instance, which points to the same allocation on the heap as the
/// source `Arc`, while increasing a reference count. When the last `Arc`
/// pointer to a given allocation is destroyed, the value stored in that allocation (often
/// referred to as "inner value") is also dropped.
///
/// Shared references in Rust disallow mutation by default, and `Arc` is no
/// exception: you cannot generally obtain a mutable reference to something
/// inside an `Arc`. If you need to mutate through an `Arc`, use
/// [`Mutex`][mutex], [`RwLock`][rwlock], or one of the [`Atomic`][atomic]
/// types.
///
/// ## Thread Safety
///
/// Unlike [`Rc<T>`], `Arc<T>` uses atomic operations for its reference
/// counting. This means that it is thread-safe. The disadvantage is that
/// atomic operations are more expensive than ordinary memory accesses. If you
/// are not sharing reference-counted allocations between threads, consider using
/// [`Rc<T>`] for lower overhead. [`Rc<T>`] is a safe default, because the
/// compiler will catch any attempt to send an [`Rc<T>`] between threads.
/// However, a library might choose `Arc<T>` in order to give library consumers
/// more flexibility.
///
/// `Arc<T>` will implement [`Send`] and [`Sync`] as long as the `T` implements
/// [`Send`] and [`Sync`]. Why can't you put a non-thread-safe type `T` in an
/// `Arc<T>` to make it thread-safe? This may be a bit counter-intuitive at
/// first: after all, isn't the point of `Arc<T>` thread safety? The key is
/// this: `Arc<T>` makes it thread safe to have multiple ownership of the same
/// data, but it  doesn't add thread safety to its data. Consider
/// <code>Arc<[RefCell\<T>]></code>. [`RefCell<T>`] isn't [`Sync`], and if `Arc<T>` was always
/// [`Send`], <code>Arc<[RefCell\<T>]></code> would be as well. But then we'd have a problem:
/// [`RefCell<T>`] is not thread safe; it keeps track of the borrowing count using
/// non-atomic operations.
///
/// In the end, this means that you may need to pair `Arc<T>` with some sort of
/// [`std::sync`] type, usually [`Mutex<T>`][mutex].
///
/// # Cloning references
///
/// Creating a new reference from an existing reference-counted pointer is done using the
/// `Clone` trait implemented for [`Arc<T>`][Arc].
///
/// ```
/// use cs431_homework::Arc;
/// let foo = Arc::new(vec![1.0, 2.0, 3.0]);
/// // The two syntaxes below are equivalent.
/// let a = foo.clone();
/// let b = Arc::clone(&foo);
/// // a, b, and foo are all Arcs that point to the same memory location
/// ```
///
/// ## `Deref` behavior
///
/// `Arc<T>` automatically dereferences to `T` (via the [`Deref`] trait),
/// so you can call `T`'s methods on a value of type `Arc<T>`. To avoid name
/// clashes with `T`'s methods, the methods of `Arc<T>` itself are associated
/// functions, called using [fully qualified syntax]:
///
/// ```
/// use cs431_homework::Arc;
///
/// let my_arc = Arc::new(5);
/// let my_five = Arc::try_unwrap(my_arc).unwrap();
/// ```
///
/// `Arc<T>`'s implementations of traits like `Clone` may also be called using
/// fully qualified syntax. Some people prefer to use fully qualified syntax,
/// while others prefer using method-call syntax.
///
/// ```
/// use cs431_homework::Arc;
///
/// let arc = Arc::new(());
/// // Method-call syntax
/// let arc2 = arc.clone();
/// // Fully qualified syntax
/// let arc3 = Arc::clone(&arc);
/// ```
///
/// [`Rc<T>`]: std::rc::Rc
/// [clone]: Clone::clone
/// [mutex]: ../../std/sync/struct.Mutex.html
/// [rwlock]: ../../std/sync/struct.RwLock.html
/// [atomic]: core::sync::atomic
/// [RefCell\<T>]: core::cell::RefCell
/// [`RefCell<T>`]: core::cell::RefCell
/// [`std::sync`]: ../../std/sync/index.html
/// [`Arc::clone(&from)`]: Arc::clone
/// [fully qualified syntax]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#fully-qualified-syntax-for-disambiguation-calling-methods-with-the-same-name
///
/// # Examples
///
/// Sharing some immutable data between threads:
///
/// ```no_run
/// use cs431_homework::Arc;
/// use std::thread;
///
/// let five = Arc::new(5);
///
/// for _ in 0..10 {
///     let five = Arc::clone(&five);
///
///     thread::spawn(move || {
///         println!("{five:?}");
///     });
/// }
/// ```
///
/// Sharing a mutable [`AtomicUsize`]:
///
/// [`AtomicUsize`]: core::sync::atomic::AtomicUsize "sync::atomic::AtomicUsize"
///
/// ```no_run
/// use cs431_homework::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::thread;
///
/// let val = Arc::new(AtomicUsize::new(5));
///
/// for _ in 0..10 {
///     let val = Arc::clone(&val);
///
///     thread::spawn(move || {
///         let v = val.fetch_add(1, Ordering::SeqCst);
///         println!("{v:?}");
///     });
/// }
/// ```
///
/// See the [`rc` documentation][rc_examples] for more examples of reference
/// counting in general.
///
/// [rc_examples]: std::rc#examples
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
    ///
    /// let mut x = Arc::new(String::new());
    /// unsafe {
    ///     Arc::get_mut_unchecked(&mut x).push_str("foo")
    /// }
    /// assert_eq!(*x, "foo");
    /// ```
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts.
        unsafe { &mut (*this.ptr.as_ptr()).data }
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
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
    /// use cs431_homework::Arc;
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
        fmt::Pointer::fmt(&(&**self), f)
    }
}
