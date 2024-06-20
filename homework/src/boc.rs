//! Concurrent Owner (Cown) type.

use core::cell::UnsafeCell;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize};
use core::{fmt, hint, ptr};
use std::sync::Arc;

/// A trait representing a `Cown`.
///
/// Instead of directly using a `Cown<T>`, which fixes _a single_ `T` we use a trait object to allow
/// multiple requests with different `T`s to be used with the same cown.
///
/// # Safety
///
/// `last()` should actually return the last request for the corresponding cown.
unsafe trait CownBase: Send {
    /// Return a pointer to the tail of this cown's request queue.
    fn last(&self) -> &AtomicPtr<Request>;
}

/// A request for a cown.
pub struct Request {
    /// Pointer to the next scheduled behavior.
    next: AtomicPtr<Behavior>,
    /// Is this request scheduled?
    scheduled: AtomicBool,
    /// The cown that this request wants to access.
    ///
    /// This is an `Arc` as the all exposed `CownPtr`s may have been dropped while the behavior is
    /// still scheduled.
    target: Arc<dyn CownBase>,
}

// SAFETY: In the basic version of BoC, user cannot get shared reference through the [`CownBase`],
// so `Sync` bound on it is not necessary.
unsafe impl Send for Request {}

impl Request {
    /// Creates a new Request.
    fn new(target: Arc<dyn CownBase>) -> Request {
        Request {
            next: AtomicPtr::new(ptr::null_mut()),
            scheduled: AtomicBool::new(false),
            target,
        }
    }

    /// Start the first phase of the 2PL enqueue operation.
    ///
    /// Enqueues `self` onto the `target` cown. Returns once all previous behaviors on this cown has
    /// finished enqueueing on all of its required cowns. This ensures the 2PL protocol.
    ///
    /// # SAFETY
    ///
    /// `behavior` must be a valid raw pointer to the behavior for `self`, and this should be the
    /// only enqueueing of this request and behavior.
    unsafe fn start_enqueue(&self, behavior: *const Behavior) {
        todo!()
    }

    /// Finish the second phase of the 2PL enqueue operation.
    ///
    /// Sets the scheduled flag so that subsequent behaviors can continue the 2PL enqueue.
    ///
    /// # Safety
    ///
    /// All enqueues for smaller requests on this cown must have been completed.
    unsafe fn finish_enqueue(&self) {
        todo!()
    }

    /// Release the cown to the next behavior.
    ///
    /// Called when `self` has been completed, and thus can allow the next waiting behavior to run.
    /// If there is no next behavior, then the cown's tail pointer is set to null.
    ///
    /// # Safety
    ///
    /// `self` must have been actually completed.
    unsafe fn release(&self) {
        todo!()
    }
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        #[allow(warnings)]
        Arc::as_ptr(&self.target).cmp(&Arc::as_ptr(&other.target))
    }
}
impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), core::cmp::Ordering::Equal)
    }
}
impl Eq for Request {}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("next", &self.next)
            .field("scheduled", &self.scheduled)
            .finish()
    }
}

/// The value should only be accessed inside a `when!` block.
#[derive(Debug)]
struct Cown<T: Send> {
    /// MCS lock tail.
    ///
    /// When a new node is enqueued, the enqueuer of the previous tail node will wait until the
    /// current enqueuer sets that node's `.next`.
    last: AtomicPtr<Request>,
    /// The value of this cown.
    value: UnsafeCell<T>,
}

// SAFETY: `self.tail` is indeed the actual tail.
unsafe impl<T: Send> CownBase for Cown<T> {
    fn last(&self) -> &AtomicPtr<Request> {
        &self.last
    }
}

/// Public interface to Cown.
#[derive(Debug)]
pub struct CownPtr<T: Send> {
    inner: Arc<Cown<T>>,
}

// SAFETY: In the basic version of BoC, user cannot get `&T`, so `Sync` is not necessary.
unsafe impl<T: Send> Send for CownPtr<T> {}

impl<T: Send> Clone for CownPtr<T> {
    fn clone(&self) -> Self {
        CownPtr {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Send> CownPtr<T> {
    /// Creates a new Cown.
    pub fn new(value: T) -> CownPtr<T> {
        CownPtr {
            inner: Arc::new(Cown {
                last: AtomicPtr::new(ptr::null_mut()),
                value: UnsafeCell::new(value),
            }),
        }
    }
}

type BehaviorThunk = Box<dyn FnOnce() + Send>;

/// Behavior that captures the content of a when body.
struct Behavior {
    /// The body of the Behavior.
    thunk: BehaviorThunk,
    /// Number of not-yet enqueued requests.
    count: AtomicUsize,
    /// The requests for this behavior.
    requests: Vec<Request>,
}

impl Behavior {
    /// Schedules the Behavior.
    ///
    /// Performs two phase locking (2PL) over the enqueuing of the requests.
    /// This ensures that the overall effect of the enqueue is atomic.
    fn schedule(self) {
        todo!()
    }

    /// Resolves a single outstanding request for `this`.
    ///
    /// Called when a request for `this` is at the head of the queue for a particular cown. If it is
    /// the last request, then the thunk is scheduled.
    ///
    /// # Safety
    ///
    /// `this` must be a valid behavior.
    unsafe fn resolve_one(this: *const Self) {
        todo!()
    }
}

impl fmt::Debug for Behavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Behavior")
            .field("thunk", &"BehaviorThunk")
            .field("count", &self.count)
            .field("requests", &self.requests)
            .finish()
    }
}

// TODO: terminator?
impl Behavior {
    fn new<C, F>(cowns: C, f: F) -> Behavior
    where
        C: CownPtrs + Send + 'static,
        F: for<'l> Fn(C::CownRefs<'l>) + Send + 'static,
    {
        todo!()
    }
}

/// Trait for a collection of `CownPtr`s.
///
/// Users pass `CownPtrs` to `when!` clause to specify a collection of shared resources, and such
/// resources can be accessed via `CownRefs` inside the thunk.
///
/// # Safety
///
/// `requests` should actually return the requests for the corresponding cowns.
pub unsafe trait CownPtrs {
    /// Types for references corresponding to `CownPtrs`.
    type CownRefs<'l>
    where
        Self: 'l;

    /// Returns a collection of `Request`.
    // This could return a `Box<[Request]>`, but we use a `Vec` to avoid possible reallocation in
    // the implementation.
    fn requests(&self) -> Vec<Request>;

    /// Returns mutable references of type `CownRefs`.
    ///
    /// # Safety
    ///
    /// Must be called only if it is safe to access the shared resources.
    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l>;
}

unsafe impl CownPtrs for () {
    type CownRefs<'l> = ()
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        Vec::new()
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {}
}

unsafe impl<T: Send + 'static, Ts: CownPtrs> CownPtrs for (CownPtr<T>, Ts) {
    type CownRefs<'l> = (&'l mut T, Ts::CownRefs<'l>)
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        let mut rs = self.1.requests();
        let cown_base: Arc<dyn CownBase> = self.0.inner.clone();
        rs.push(Request::new(cown_base));
        rs
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {
        unsafe { (&mut *self.0.inner.value.get(), self.1.get_mut()) }
    }
}

unsafe impl<T: Send + 'static> CownPtrs for Vec<CownPtr<T>> {
    type CownRefs<'l> = Vec<&'l mut T>
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        self.iter().map(|x| Request::new(x.inner.clone())).collect()
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {
        self.iter()
            .map(|x| unsafe { &mut *x.inner.value.get() })
            .collect()
    }
}

/// Creates a `Behavior` and schedules it. Used by "When" block.
pub fn run_when<C, F>(cowns: C, f: F)
where
    C: CownPtrs + Send + 'static,
    F: for<'l> Fn(C::CownRefs<'l>) + Send + 'static,
{
    Behavior::new(cowns, f).schedule();
}

/// from <https://docs.rs/tuple_list/latest/tuple_list/>
#[macro_export]
macro_rules! tuple_list {
    () => ( () );

    // handling simple identifiers, for limited types and patterns support
    ($i:ident)  => ( ($i, ()) );
    ($i:ident,) => ( ($i, ()) );
    ($i:ident, $($e:ident),*)  => ( ($i, $crate::tuple_list!($($e),*)) );
    ($i:ident, $($e:ident),*,) => ( ($i, $crate::tuple_list!($($e),*)) );

    // handling complex expressions
    ($i:expr)  => ( ($i, ()) );
    ($i:expr,) => ( ($i, ()) );
    ($i:expr, $($e:expr),*)  => ( ($i, $crate::tuple_list!($($e),*)) );
    ($i:expr, $($e:expr),*,) => ( ($i, $crate::tuple_list!($($e),*)) );
}

/// "When" block.
#[macro_export]
macro_rules! when {
    ( $( $cs:ident ),* ; $( $gs:ident ),* ; $thunk:expr ) => {{
        run_when(tuple_list!($($cs.clone()),*), move |tuple_list!($($gs),*)| $thunk);
    }};
}

#[test]
fn boc() {
    let c1 = CownPtr::new(0);
    let c2 = CownPtr::new(0);
    let c3 = CownPtr::new(false);
    let c2_ = c2.clone();
    let c3_ = c3.clone();

    let (finish_sender, finish_receiver) = crossbeam_channel::bounded(0);

    when!(c1, c2; g1, g2; {
        // c3, c2 are moved into this thunk. There's no such thing as auto-cloning move closure.
        *g1 += 1;
        *g2 += 1;
        when!(c3, c2; g3, g2; {
            *g2 += 1;
            *g3 = true;
        });
    });

    when!(c1, c2_, c3_; g1, g2, g3; {
        assert_eq!(*g1, 1);
        assert_eq!(*g2, if *g3 { 2 } else { 1 });
        finish_sender.send(()).unwrap();
    });

    // wait for termination
    finish_receiver.recv().unwrap();
}

#[test]
fn boc_vec() {
    let c1 = CownPtr::new(0);
    let c2 = CownPtr::new(0);
    let c3 = CownPtr::new(false);
    let c2_ = c2.clone();
    let c3_ = c3.clone();

    let (finish_sender, finish_receiver) = crossbeam_channel::bounded(0);

    run_when(vec![c1.clone(), c2.clone()], move |mut x| {
        // c3, c2 are moved into this thunk. There's no such thing as auto-cloning move closure.
        *x[0] += 1;
        *x[1] += 1;
        when!(c3, c2; g3, g2; {
            *g2 += 1;
            *g3 = true;
        });
    });

    when!(c1, c2_, c3_; g1, g2, g3; {
        assert_eq!(*g1, 1);
        assert_eq!(*g2, if *g3 { 2 } else { 1 });
        finish_sender.send(()).unwrap();
    });

    // wait for termination
    finish_receiver.recv().unwrap();
}
