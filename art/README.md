# Adaptive radix tree (ART)

Implement the data structure described in the following paper: Leis *et. al.* [The Adaptive Radix
Tree: ARTful Indexing for Main-Memory Databases](https://db.in.tum.de/~leis/papers/ART.pdf).  ICDE
2013.

<!-- Leis *et. al.* [The ART of Practical
Synchronization](https://db.in.tum.de/~leis/papers/artsync.pdf).  DaMoN 2016. -->


## Specification

- **Correctness**: your `Art` should behave exactly the same as `HashMap<String, V>`.  See
  `stress()` in `tests/art_test.rs` for more details.

    + More test cases will be added for giving partial credits to partial solutions.

- **Performance**: it doesn’t matter as far as your implementation is "reasonably" fast.

- **No shrinking**: You don’t need to shrink a node even if it’s underfull.

- **Path compression**: implement the "pessimistic" path compression optimization described in the
  paper. Hint: it's already implemented in the skeleton.

- **No cheating**: You should not "cheat", e.g. using `HashMap` or other existing data structures to
  mimic the behavior of `Art`. In particular, don’t change the given `Art` data type.

- **Submission**: You can only edit and submit `art.rs` in the skeleton code. If you feel the other
  parts of the skeleton code should be changed (e.g. adding a dependency in `Cargo.toml`), ask such
  a change in the issue tracker. Submission method is TBA.


## Recommendations

- Read the paper and the skeleton code carefully.  I'll ask questions on those in the exams.

- Homework 2: concurrent adaptive radix tree will be based on this homework.

- Read [the Rust book](https://doc.rust-lang.org/book/), especially the ["getting started"
  section](https://doc.rust-lang.org/book/ch01-00-getting-started.html) for learning how to build
  and test the development.

- Use [Visual Studio Code](https://code.visualstudio.com/) of
  [CLion](https://www.jetbrains.com/clion/) for interactive debugging.  The former is free of charge
  for everyone, and The latter is [free of charge for students](https://www.jetbrains.com/student/).
    + [Manual for debugging rust code in
      VSCode](https://www.forrestthewoods.com/blog/how-to-debug-rust-with-visual-studio-code/)
      (using [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
      plugin)
    + [Manual for debugging rust code in
      CLion](https://www.jetbrains.com/help/clion/rust-support.html)

- The following commands will help you in debugging your code:
    + Testing: `cargo test`
    + Stress testing: `cargo test stress`
    + Stress testing with address sanitizer enabled: `RUSTFLAGS="-Z sanitizer=address" cargo
      +nightly test stress --target x86_64-unknown-linux-gnu`
