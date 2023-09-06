# Concurrent set based on Lock-coupling linked list
**Implement concurrent set data structures with sorted singly linked list using (optimistic) fine-grained lock-coupling.**

Suppose you want a set data structure that supports concurrent operations.
The simplest possible approach would be taking a non-concurrent set implementation and protecting it with a global lock.
However, this is not a great idea if the set is accessed frequently because a thread's operation blocks all the other threads' operations.

In this homework, you will write two implementations of the set data structures based on singly linked list protected by fine-grained locks.
* The nodes in the list are sorted by their value, so that one can efficiently check if a value is in the set.
* Each node has its own lock that protects its `next` field.
  When traversing the list, the locks are acquired and released in the hand-over-hand manner.
  This allows multiple operations run more concurrently.

You will implement two variants.
* In `list_set/fine_grained.rs`, the lock is the usual `Mutex`.
* In `list_set/optimistic_fine_grained.rs`, the lock is `SeqLock`.
  This allows read operations to run optimistically without actually locking.
  Therefore, read operations are more efficient in read-most scenario, and
  they do not block other operations.
  However, you need to take more care to get it correct.
    * You need to validate read operations and handle the failure.
    * Since reads and writes can run concurrently, you should use atomic operations to avoid data race.
      Specifically, you will use `crossbeam_epoch`'s `Atomic<T>` type.
      For `Ordering`, use `SeqCst` everywhere.
      In the later part of this course, you will learn that `Relaxed` is sufficient.
      But don't use `Relaxed` in this homework, because that would break `cargo_tsan`.
    * Since a node can be removed while another thread is reading,
      reclamation of the node should be deferred.
      Again, you will need to use `crossbeam_epoch`.

Fill in the `todo!()`s in `list_set/{fine_grained,optimistic_fine_grained}.rs` (about 40 + 80 lines of code).
As in the [Linked List homework](./linked_list.md), you will need to use some unsafe operations.


## Grading (100 points)
Run
```
./scripts/grade-list_set.sh
```

For each module `fine_grained` and `optimistic_fine_grained`,
the grader runs the following tests with `cargo`, `cargo_asan`, and `cargo_tsan` in the following order.
1. `stress_sequential` (5 points)
1. `stress_concurrent` (10 points)
1. `log_concurrent` (15 points)
1. `iter_consistent` (15 points)

For `optimistic_fine_grained`, the grade additionally runs
1. `read_no_block` (5 points)
1. `iter_invalidate_end` (5 points)

## Submission
```sh
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-list_set.zip
```

Submit `list_set.zip` to gg.
