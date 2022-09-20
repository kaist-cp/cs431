# Hazard pointers
Implement Hazard Pointers.

1. Read [this paper](https://ieeexplore.ieee.org/document/1291819).
   While this paper is sufficient for understanding Hazard Pointers,
   you may also want to take a look at [WG21 P1121](http://wg21.link/P1121),
   the proposal for adding Hazard Pointers to the C++ standard library.
1. Read the documentation in [`hazard_pointer/mod.rs`](../src/hazard_pointer/mod.rs)
1. Fill in the `todo!()`s in
   [`hazard_pointer/hazard.rs`](../src/hazard_pointer/hazard.rs) and
   [`hazard_pointer/retire.rs`](../src/hazard_pointer/retire.rs)
   (approx 75 lines).
1. Implement using the `SeqCst` ordering for every atomic accesses first, and then use more relaxed orderings.
   In particular, you will need to add two `fence`s.

## Grading (100 points)
Run `./scripts/grade-hazard_pointer.sh`.

## Testing correctness with `SeqCst` ordering
Like [hash table](./hash_table.md), we will first test if your implementation with `SeqCst` ordering is correct.
* tested with `cargo[_asan,_tsan] [--release]`
    * tests in `hazard.rs` (20 points)
    * test in `retire.rs` (10 points)
    * tests in `tests/hazard_pointer.rs` (40 points)

## Testing correctness with relaxed orderings
Like [arc](./arc.md), we will additionally use loom to test your hazard pointer implementation with relaxed orderings.
* tested with `cargo --features check-loom`
    * tests in `tests/hazard_pointer.rs` `mod sync` (30 points)

Note that we will also run the tests used in [`SeqCst` ordering](#testing-correctness-with-seqcst-ordering) as well,
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

Currently, loom does not understand `get_mut()`.
Please use `load()` with `Ordering::Relaxed` as an alternative that loom is able to understand.
