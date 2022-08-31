# Hazard Pointers
Implement Hazard Pointers.

1. Read [this paper](https://ieeexplore.ieee.org/document/1291819).
   While this paper is sufficient for understanding Hazard Pointers,
   you may also want to take a look at [WG21 P1121](http://wg21.link/P1121),
   the proposal for adding Hazard Pointers to the C++ standard library.
2. Read the documentation in [`hazard_pointer/mod.rs`](../src/hazard_pointer/mod.rs)
3. Fill in the `todo!()`s in
   [`hazard_pointer/hazard.rs`](../src/hazard_pointer/hazard.rs) and
   [`hazard_pointer/retire.rs`](../src/hazard_pointer/retire.rs)
   (approx 75 lines).

## Grading (100 points)
Run `./scripts/grade-6.sh`.

Grading scheme
* tested with `cargo[_asan] [--release]`
    * tests in `hazard.rs` (20 points)
    * test in `retire.rs` (10 points)
    * tests in `tests/hazard_pointer.rs` (40 points)
* tested with `cargo --features check-loom`
    * tests in `tests/hazard_pointer.rs` `mod sync` (30 points)

## Submission
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw6.zip
```
Submit `hw6.zip` to gg.
