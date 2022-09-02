# Lock-free hashtable
**Implement lock-free hash table based on recursive split-ordered list.**

## Guide
1. Fully understand the following reading material.
    + [The original paper on split-ordered list](https://dl.acm.org/doi/abs/10.1145/1147954.1147958).
      You can skip the correctness proof and performance evaluation section.
    + Chapter 13.3 of [The Art of Multiprocessor Programming](https://www.amazon.com/Art-Multiprocessor-Programming-Revised-Reprint/dp/0123973376):
      Presents the same stuff, but more readable.
      [pdf](https://dl.acm.org/doi/book/10.5555/2385452) of the book can be downloaded for free in KAIST.
    + The [lock-free linked list](https://github.com/kaist-cp/cs431/blob/main/lockfree/src/list.rs) interface and implementation.
1. Implement `GrowableArray` in [`hash_table/growable_array.rs`](../src/hash_table/growable_array.rs). (about 100 LOC)
    * You'll need to perform integer-pointer casts because `AtomicUsize` in `Segment` can be interpreted as both `Atomic<T>` and `Atomic<Segment>`.
      Use [`into_usize` and `from_usize`](https://docs.rs/crossbeam/*/crossbeam/epoch/trait.Pointer.html).
      See also: [#225](https://github.com/kaist-cp/cs431/issues/225)
    * To represent the height of the segment tree, [tag](https://en.wikipedia.org/wiki/Tagged_pointer) the `root` pointer with the height.
      Use [`tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.tag) and [`with_tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.with_tag).
      See [lock-free list](https://github.com/kaist-cp/cs431/blob/main/lockfree/src/list.rs) for example usage.
      See also: [#226](https://github.com/kaist-cp/cs431/issues/226)
1. Implement `SplitOrderedList` in [`hash_table/split_ordered_list.rs`](../src/hash_table/split_ordered_list.rs). (about 80 LOC)
    * You can use bitwise operations on `usize` e.g. `<<`, `&`, `|`, `^`, ...
      See also: [`leading_zeros`](https://doc.rust-lang.org/std/primitive.usize.html#method.leading_zeros), [`reverse_bits`](https://doc.rust-lang.org/std/primitive.usize.html#method.reverse_bits), [`size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html)
    * We provided type signatures for 2 helper methods for `SplitOrderedList`.
      You can modify/remove them or add more private methods if you want to.
      Just make sure you don't change the public interface. You can import other stuff from the `core` or `crossbeam_epoch` crates (but not necessary).

## Testing
Tests in `tests/{growable_array,hash_table}.rs` uses the map test functions defined in `tests/map/mod.rs`.
* `smoke`:
  Simple test case that tries a few operations. Useful for debugging.
* `stress_sequential`:
  Runs many operations in a single thread and tests if it works like a map data structure using `std::collections::HashMap` as reference.
* `lookup_concurrent`:
  Inserts keys sequentially, then concurrently runs lookup in multiple threads.
* `insert_concurrent`:
  Inserts concurrently.
* `stress_concurrent`:
  Randomly runs many operations concurrently.
* `log_concurrent`:
  Randomly runs many operations concurrently and logs the operations & results per thread.
  Then checks the consistency of the log.
  For example, if the key `k` was successfully deleted twice, then `k` must have been inserted at least twice.
  This check doesn't guarantee complete correctness unlike `stress_sequential`.

## Grading (180 points)
Run `./scripts/grade-hash_table.sh`.

For each module `growable_array` and `split_ordered_list`,
the grader runs the tests with `cargo`, `cargo_asan`, and `cargo_tsan` in the following order.
1. `stress_sequential` (10 points)
2. `lookup_concurrent` (10 points)
3. `insert_concurrent` (10 points)
4. `stress_concurrent` (30 points)
5. `log_concurrent` (30 points)

Note:
* If a test fails in a module, then the later tests in the same module will not be run.
* The test timeout is at least 5x of the time our implementation took on the homework server.
  It is not a tight timeout, but it will detect implementations that are clearly incorrect.


## Submission
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw5.zip
```
Submit `hw5.zip` to gg.


## FAQ

### Is ThreadSanitizer check meaningful for lock-free data structure?
> If I understand correctly, role of the thread sanitizer is to detect data race, which can be prevented by using locks.
> I think that the data race is inevitable in lock-free data structures since they are "lock-free".
> And I believe that the goal of lock-free data structure is to guarantee the correctness of data structure,
> even in the situation where data race inevitably exists and multiple thread access the data in arbitrary order.

We say there is a data race if multiple (unsynchronized) threads are accessing a location and at least one access is write.
Clearly lock-free data structures have data race in that regard.
However, not all data races are created equal.
Some races are inherent and expected like lock-free data structures, and the other races are totally unexpected
(e.g. a variable that should be protected by lock is accessed without holding a lock).
The job of tools like ThreadSanitizer is to detect the latter type of race.
But how do we tell the tools that a race is actually expected?
We use the *atomic* operations like `AtomicUsize::store`.
All the other operations are considered non-atomic,
e.g. `=` (this actually is atomic at the architecture level for types like `usize`, but considered non-atomic in this context), `memcpy`, etc,
and races caused by them are considered real.
[In fact, C/C++ (which Rust follows) defines data race as a conflict caused by non-atomic operations](https://en.cppreference.com/w/cpp/language/memory_model#Threads_and_data_races).

Related:
* The problem of ThreadSanitizer is that it doesn't understand synchronization using fences. Because of this, it fails to establish happens-before relation between non-atomic operations, which leads to false positive report. This is not a big problem for most programs because they will simply use locks.
* In **safe** Rust, the type system guarantees data race freedom by the exclusiveness of `&mut` and  `Send`/`Sync` traits.
