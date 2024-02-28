# Concurrent set based on Lock-coupling linked list
**Implement concurrent set data structures with sorted singly linked list using fine-grained lock-coupling.**

Suppose you want a set data structure that supports concurrent operations.
The simplest possible approach would be taking a non-concurrent set implementation and protecting it with a global lock.
However, this is not a great idea if the set is accessed frequently because a thread's operation blocks all the other threads' operations.

In this homework, you will write an implementation of the set data structure based on singly linked list protected by fine-grained locks.
* The nodes in the list are sorted by their value, so that one can efficiently check if a value is in the set.
* Each node has its own lock that protects its `next` field.
  When traversing the list, the locks are acquired and released in the hand-over-hand manner.
  This allows multiple operations run more concurrently.

Fill in the `todo!()`s in `list_set/fine_grained.rs` (about 40 lines of code).
As in the [Linked List homework](./linked_list.md), you will need to use some unsafe operations.

## Testing
Tests are defined in `tests/list_set/fine_grained.rs`.
Some of them use the common set test functions defined in `src/test/adt/set.rs`.

## Grading (45 points)
Run
```
./scripts/grade-list_set.sh
```

The grader runs the tests
with `cargo`, `cargo_asan`, and `cargo_tsan` in the following order.
1. `stress_sequential` (5 points)
1. `stress_concurrent` (10 points)
1. `log_concurrent` (15 points)
1. `iter_consistent` (15 points)

For the above tests, if a test fails in a module, then the later tests in the same module will not be run.

## Submission
```sh
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-list_set.zip
```

Submit `hw-list_set.zip` to gg.

## Advanced (optional)
**Note**: This is an *optional* homework, meaning that it will not be graded and not be asked in the exam.

Consider a variant of the homework that uses `SeqLock` instead of `Mutex`.
This allows read operations to run optimistically without actually locking.
Therefore, read operations are more efficient in read-most scenario, and
they do not block other operations.
However, more care must be taken to ensure correctness.
  * You need to validate read operations and handle the failure.
      * Do not use `ReadGuard::restart()`.
        Using this correctly requires some extra synchronization
        (to be covered in lock-free list lecture),
        which makes `SeqLock` somewhat pointless.
        The tests assume that `ReadGuard::restart()` is not used.
  * Since each node can be read and modified to concurrently,
    you should use atomic operations to avoid data races.
    Specifically, you will use `crossbeam_epoch`'s `Atomic<T>` type
    (instead of `std::sync::AtomicPtr<T>`, due to the next issue).
    For `Ordering`, use `SeqCst` everywhere.
    (In the later part of this course, you will learn that `Relaxed` is sufficient.
    But don't use `Relaxed` in this homework, because that would break `cargo_tsan`.)
  * Since a node can be removed while another thread is reading,
    reclamation of the node should be deferred.
    You can handle this semi-automatically with `crossbeam_epoch`.

**Instruction**: Fill in the `todo!()`s in `list_set/optimistic_fine_grained.rs` (about 80 lines of code).

**Testing**: Tests are defined in `tests/list_set/optimistic_fine_grained.rs`.

**Self grading**:
Run
```
./scripts/grade-optimistic_list_set.sh
```

Unlike the main homework, the grader additionally runs the following tests
(10 points if all of them passes, otherwise 0).
* `read_no_block`
* `iter_invalidate_end`
* `iter_invalidate_deleted`
