# Hazard pointers
**Implement Hazard Pointers.**

Fill in the `todo!()`s in
[`hazard_pointer/hazard.rs`](../src/hazard_pointer/hazard.rs) and
[`hazard_pointer/retire.rs`](../src/hazard_pointer/retire.rs)
(approx 75 lines).

This homework is in 2 parts:
1. (70 points) Functionality:
   Implement hazard pointers with `Ordering::SeqCst` for every atomic access.
   With this, we can pretend that we are on a sequentially consistent memory model.
2. (30 points) Performance:
   After you learn about relaxed memory semantics,
   optimize the implementation by relaxing the ordering.
   We recommend working on this part after finishing the [Arc homework](./arc.md).

## ***2024 spring semester notice: Part 2 is cancelled***
We won't cover the weak memory semantics in this semester.
To ensure that the grader works properly, you must use `Ordering:SeqCst` for all operations.

## Part 1: Hazard pointers in the sequentially consistent memory model

Read [this paper](https://ieeexplore.ieee.org/document/1291819).
While this paper is sufficient for understanding Hazard Pointers,
you may also want to take a look at [WG21 P2530](https://wg21.link/p2530),
the proposal for adding Hazard Pointers to the C++ standard library.
The rest of this section summarizes the algorithm and correctness argument of hazard pointers.


Suppose a data structure has a memory block b.
A thread (T1) wants to read the value written in b and
another thread (T2) wants to remove b from the data structure and free the memory.
To prevent use-after-free,
T1 has to ensure that b is not freed before reading b and
T2 has to check that no other threads are accessing b before freeing b.
The hazard pointer library implements this mechanism as follows:

```
(T1-1) Add b to the hazard list       | (T2-1) Unlink b and `retire(b)`
       (`Shield::try_protect()`)      |
(T1-2) Check if b is still reachable  | (T2-2) Check if b is in the hazard list
       if so, deref b                 |        if not, free b
(T1-3) Remove b from the hazard list  |
       (`Shield::drop()`)             |
```

To show that the algorithm prevents use-after-free,
let's consider all possible interleavings of each step
(in the sequentially consistent memory model).

First, if `T1-3 → T2-2` (`T2-2` is executed after `T1-3`),
then b is freed after all accesses.

Second, in all the remaining cases,
either `T1-1 → T2-2` or `T2-1 → T1-2` holds
(otherwise, there is a cycle `T1-1 → T1-2 → T2-1 → T2-2 → T1-1`).
- If `T1-1 → T2-2`, then b is not freed.
- If `T2-1 → T1-2`, then the validation fails, so `T1` will not dereference b.

Therefore, the algorithm is correct in the sequentially consistent memory model.


## Part 2: Relaxing the orderings

If you use `Ordering::Relaxed`,
the correctness argument from the previous section doesn't hold.
The problem is that in the relaxed memory model,
`→` ("executed before") doesn't imply that
the latter instruction sees the effect of the earlier instruction.
To fix this, we should add some synchronization operations
so that `→` implies "happens-before".

First, if `T2-2` saw the result of `T1-3`,
then we want to enforce `deref b @ T1` happens before `free b @ T2`
To enforce this,
it suffices to add release-acquire synchronization between `T1-3` and `T2-2`
(recall the synchronization in `Arc::drop`).

For the second case, release-acquire doesn't guarantee
"either `T1-1` happens before `T2-2` or `T2-1` happens before `T1-2`".
Because of that, `T1-2` may not read the message of `T2-1`
and `T2-2` may not read the message of `T1-2` at the same time,
leading to concurrent `deref b` and `free b`.
To make this work, we should insert an SC fence (`fence(SeqCst)`)
between `T1-1` and `T1-2`, and another between `T2-1` and `T2-2`.
<!-- This should be explained in the lecture.
Recall that an SC fence joins the executing thread's view and the global SC view.
This means that
the view of a thread after executing its SC fence
is entirely included in the view of another thread after its SC fence.
If we insert an SC fence between
`T1-1` and `T1-2`, and another between `T2-1` and `T2-2`,
then either `T1's fence ⊑ T2's fence` or `T2's fence ⊑ T1's fence` holds.
Therefore, `T1-1 ⊑ T2-2` or `T2-1 ⊑ T1-2`.
-->

## Grading (100 points)
Run `./scripts/grade-hazard_pointer.sh`.

### Part 1: Functionality (70 points)
Like [hash table](./hash_table.md), we will first test if your implementation with `SeqCst` ordering is correct.
* tested with `cargo[_asan,_tsan] [--release]`
    * tests in `hazard.rs` (20 points)
    * a test in `retire.rs` (10 points)
    * tests in `tests/hazard_pointer.rs` (40 points)

### Part 2: Relaxed orderings (30 points)
Like [arc](./arc.md), we will additionally use the loom model checker to test your hazard pointer implementation with relaxed orderings.
* tested with `cargo --features check-loom`
    * tests in `tests/hazard_pointer.rs` `mod sync` (30 points)

Note that we will also run the tests for part 1 as well,
so make sure your implementation still passes all tests.

## Submission
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-hazard_pointer.zip
```
Submit `hw-hazard_pointer.zip` to gg.

## FAQ

> loom throws an error when I used `get_mut()` on a `AtomicPtr`.

Currently, loom does not understand `get_mut()`
(<https://github.com/tokio-rs/loom/issues/154>).
Please use `load()` with `Ordering::Relaxed` instead.
