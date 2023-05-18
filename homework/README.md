# Tips

- Read the paper and the skeleton code carefully.  I'll ask questions on those in the exams.

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

- Use rustfmt and clippy:

  ```
  cargo fmt
  cargo clippy
  ```

- Running individual tests

  ```
  # Run all tests in a module
  cargo test --test <module name>
  # For example, run all tests in the hazard_pointer module
  cargo test --test hazard_pointer
  # Run all tests in a module
  cargo test --test <module name> <test name>
  # For example, run the stack_queue test in the hazard_pointer module
  cargo test --test hazard_pointer stack_queue
  ```

## Using LLVM Sanitizers

We are going to use the LLVM sanitizers for grading.
Sanitizers are dynamic analysis tools that detects buggy behaviors during runtime. For example,
[AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) detects memory bugs like use-after-free and
[ThreadSanitizer](https://clang.llvm.org/docs/ThreadSanitizer.html) detects data races.

You can run the tests with sanitizers using following commands:
```
source scripts/grade-utils.sh
# This may take some time because of `rustup toolchain update stable nightly` in the script.
# If you have run that already, please feel free to comment that line out.

cargo_asan SUBCOMMAND
# cargo_asan runs the following command
# RUSTFLAGS="-Z sanitizer=address" cargo +nightly SUBCOMMAND --target x86_64-unknown-linux-gnu

# For example, run all tests in the hazard_pointer module under the address sanitizer
cargo_asan test --test hazard_pointer

cargo_tsan SUBCOMMAND
# cargo_tsan runs the following command
# TSAN_OPTIONS="suppressions=suppress_tsan.txt" RUST_TEST_THREADS=1 RUSTFLAGS="-Z sanitizer=thread" cargo +nightly SUBCOMMAND --target x86_64-unknown-linux-gnu
# (`suppressions=suppress_tsan.txt` is for suppressing some false positive from ThreadSanitizer.)

# For example, run all tests in the growable_array module under the thread sanitizer
cargo_tsan test --test growable_array
```

While (safe) Rust's type system guarantees memory safety and absence of data race,
this guarantee relies on the correctness of the libraries implemented with unsafe features.
Therefore tools like sanitizers are still essential when we use unsafe Rust.
