# Core Arc
**Implement a simplified version of `Arc` without support for `Weak`.**

In this homework, you will practice release-acquire synchronization in weak memory
by implementing a simplified version of `Arc`.
Specifically, you will use atomic operations of `AtomicUsize`
to synchronize the accesses to the underlying data.

Fill in the `todo!()`s in `src/arc.rs`.
The total lines of code to be written is about 25.
The skeleton code is a heavily modified version of `Arc` from the standard library.
We don't recommend reading the original source code before finishing this homework
because that version is more complex.

## ***2024 spring semester notice: Use `SeqCst`***
We won't cover the weak memory semantics in this semester.
So you may ignore the instructions on `Ordering` stuff below and
use `Ordering::SeqCst` for `ordering: Ordering` parameters for `std::sync::atomic` functions.

## Guide

Follow [the Arc section of the Rustnomicon (the book on unsafe Rust)][nomicon-arc].

Some food for thought on Rustnomicon's description:
* Quiz: Why does `Arc<T> : Sync` require `T : Send`?
* The [Layout section](https://doc.rust-lang.org/nomicon/arc-mutex/arc-layout.html) explains
  why [`NonNull`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html) and `PhantomData` are necessary.
  We don't care about them in this course and will not ask about them in the exams
  (it's quite interesting, though).
* Their implementation uses `fence(Acquire)`, which we may not be able cover in the lecture due to time constraints.
  You can implement (a slightly inefficient version of) Arc only with `AtomicUsize`'s methods and the concepts we covered in the lecture
  (you will need to use `Ordering::AcqRel` in some places).
  Using `fence(Acquire)` is not required in the homework and exam.
  If you want to fully understand the `fence(Acquire)` version,
  read §4 of the [Promising semantics paper](https://sf.snu.ac.kr/publications/promising.pdf).


### Synchronization requirements of `Arc`

To ensure that data race does not occur in the implementation of Arc and the clients of Arc,
add enough synchronization operations to the Arc implementation:
* The initialization of the `ArcInner` memory block (in `new()`) happens before the accesses of its fields.
  (Guaranteeing this doesn't require extra synchronization operation in Arc. Quiz: Why?)
* Accesses to the fields of `ArcInner` happen before the deallocation of the `ArcInner` memory block (in the last `drop()`).
* Non-atomic writes to the data (via `&mut T` from `get_mut()`, `make_mut()`, `try_unwrap()`) happen after/before all the other accesses (via `&T` from `deref()`).
  More strictly, `&mut T` to the data must not concurrently coexist with `&T` (Rust's aliasing rule).


<!-- ## Grading (50 points) -->
## Grading (40 points)
Run `./scripts/grade-arc.sh`.

1. Functionality (25):
   First, the grader will check if
   your implementation passes the doc tests and the tests in `tests/arc.rs`.
   You can manually re-run the test with the following commands:
    ```
    cargo test --test arc
    cargo test --doc arc
    source scripts/grade-utils.sh
    cargo_asan test --test arc
    cargo_asan test --doc arc
    ```
<!-- 1. Correctness (25): -->
1. Correctness (15):
   Then the grader runs the tests with
   [the Loom model checker](https://github.com/tokio-rs/loom)
   to check all possible executions (interleaving & reordering) in the memory model.
   <!--
   If your code doesn't pass these tests,
   then you need to add more synchronization operations or
   fix the memory ordering of them.
   -->
   You can manually re-run the tests with this command.
    ```
    cargo test --features check-loom --test arc -- --nocapture --test-threads 1
    ```
<!--
1. Efficiency:
   Make sure that you don't use `SeqCst` ordering.
   No points will be given if your solution contains `SeqCst`.
   We will not check if your implementation is optimal in terms of synchronization,
   but we encourage you to find the minimal set of synchronization operations.
-->


## Submission
Submit `arc.rs` to gg.


## Other Tips
* Read
  [`sync::atomic::AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html) and
  [`sync::Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html).
  The semantics covered in the lectures applies to these.
* You may need to use
  [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)
  in `try_unwrap`.
* If the test failure message is not descriptive enough,
  try adding `-- --nocapture --test-threads 1`.

### FAQ: AddressSanitizer reports a memory leak in my implementation.
It might be the case that
you're not deallocating the heap memory block in your `Drop` implementation.
For example, if you call functions like `drop_in_place` on `*mut ArcInner<_>`,
it only runs the destructor of `ArcInner`
without freeing the memory where that `ArcInner` lived.

The standard method to free the heap memory block is to convert the pointer
`*mut T` to `Box<T>` whose destructor runs the destructor of `T` and frees the
heap memory occupied by `T`.
For example, `pop_front_node` from HW2 uses `Box::from_raw` to convert the head
pointer into `Box<Node<_>>` and dereferences it,
moving it out of the heap to a temporary location and freeing the memory block in the heap.

For more information, see
<https://github.com/kaist-cp/cs431/issues/125>,
<https://doc.rust-lang.org/reference/destructors.html>, and
<https://doc.rust-lang.org/std/boxed/index.html>.


[nomicon-arc]: https://doc.rust-lang.org/nomicon/arc-mutex/arc.html
[ORC11]: https://plv.mpi-sws.org/rustbelt/rbrlx/
