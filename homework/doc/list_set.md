# Concurrent set based on Lock-coupling linked list
**Implement a concurrent set data structure with sorted singly linked list using fine-grained lock-coupling.**

Suppose you want a set data structure that supports concurrent operations.
The simplest possible approach would be taking a non-concurrent set implementation and protecting it with a global lock.
However, this is not a great idea if the set is accessed frequently because a thread's operation blocks all the other threads' operations.

In this homework, we aim to implement the set using a linked list that uses fine-grained locks to allow multiple threads to operate on the set.
In this data structure, each node of the list has its lock,
and the operations traverse the list while acquiring the lock of the next node and releasing the lock of the previous node.
Although a thread holding a node lock may still block another thread that wants to acquire that lock,
it allows operations to run more concurrently than using a global lock.

For better performance, the list should keep the elements sorted.
Without this, maintaining the uniqueness of each element in the set requires that all the operations traverse to the end of the list to check if the element of interest is already in the list.
By keeping the list sorted, each operation only needs to traverse to the given key.

Your job is to fill in `todo!()`s in `list_set.rs` (about 45 lines of code). As in the [Linked List homeowork](./linked_list.md), you will need to use some unsafe operations.

## Grading
Run
```
./scripts/grade-list_set.sh
```

Grading scheme:
* +10 points if `cargo test [--release]` tests pass.
* +70 points if `cargo_asan [--release]` and `cargo_tsan [--release]` tests pass.
    * To manually run `cargo_asan` and `cargo_tsan`, first run `source ./scripts/grade-util.sh`.

## Submission
Submit `list_set.rs` to gg.
