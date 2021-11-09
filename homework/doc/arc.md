# Core Arc
**Implement a simplified version of `Arc` without the support for `Weak`.**

In this homework, you will practice basic synchronization in the relaxed memory model by implementing a simplified version of `Arc`.  Specifically, you will use atomic operations of `AtomicUsize` and/or `fence` to synchronize the accesses to the underlying data with its deallocation.  Read the documentation in `src/arc.rs` carefully and fill in the `todo!()`s.  The total lines of code to be written is about 25.

The skeleton code is heavily modified version of `Arc` from the standard library.  We don't recommend reading the original source code before finishing this homework because that version is more complex.

## Grading (50 points)
Run `./scripts/grade-3.sh`.

1. Functionality (25): First, the grader will check if your implementation passes the doc tests and the tests in `tests/arc.rs`. You can manually re-run the test with following commands:
    ```
    cargo test --test arc
    cargo test --doc arc
    RUSTFLAGS="-Z sanitizer=address" RUSTDOCFLAGS="-Z sanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu --test arc
    RUSTFLAGS="-Z sanitizer=address" RUSTDOCFLAGS="-Z sanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu --doc arc
    ```
2. Correctness (25): Then the grader runs the tests with [the Loom model checker](https://github.com/tokio-rs/loom) to check all possible executions (interleaving & reordering) in the memory model. If your code doesn't pass these tests, then you need to add more synchronization operations or fix the memory ordering of them. You can manually re-run the tests with this command.
    ```
    cargo test --features check-loom --test arc -- --nocapture --test-threads 1
    ```
3. Efficiency: Make sure that you don't use `SeqCst` ordering. We'll give 0 point if your solution contains `SeqCst`. We will not check if your implementation is optimal in terms of synchronization, but we encourage you to find the minimal set of synchronization operations.

## Submission
Submit `arc.rs` to gg.

## Tips
* *The Rustonomicon* has [a section on implementing a simpler version of Arc](https://doc.rust-lang.org/nomicon/arc-mutex/arc.html). It is very similar to this homework, so you can use it as a starting point if that helps. But make sure that you understand the reasoning behind the synchronization requirement and implementation. We may ask about them in the exam!
* Read [`std::sync::atomic::AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html), [`std::sync::atomic::fence`](https://doc.rust-lang.org/std/sync/atomic/fn.fence.html), and [`std::sync::Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html). The semantics covered in the lectures applies to these.
* [`std::ptr::NonNull`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html) is basically a raw pointer that is always non-null (+ some subtle properties that we don't care about in this course). The only method you need in this homework is [`as_ptr`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html#method.as_ptr).
* You may need to use [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html) in `try_unwrap`.
* If the test failure message is not descriptive enough, try adding `-- --nocapture --test-threads 1`.

### FAQ: AddressSanitizer reports memory leak in my Arc implementation.
It might be the case that you're not actually deallocating the heap memory block in your `Drop` implementation. For example, if you call functions like `drop_in_place` on `*mut ArcInner<_>`, it only runs the destructor of `ArcInner` without freeing the memory where that `ArcInner` lived.

The standard method to free the heap memory block is to convert the pointer `*mut T` to `Box<T>` whose destructor runs the destructor of `T` and frees the heap memory occupied by `T`. For example, `pop_front_node` from HW2 uses `Box::from_raw` to convert the head pointer into `Box<Node<_>>` and returns that box. When this box gets dropped, the destructor of box will free the memory block of the node.

For more information, see https://github.com/kaist-cp/cs431/issues/125, https://doc.rust-lang.org/reference/destructors.html, and https://doc.rust-lang.org/std/boxed/index.html.
