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

- The following commands will help you in debugging your code:
    + Testing with address sanitizer enabled:
      ```
      RUSTFLAGS="-Z sanitizer=address" cargo +nightly test TEST_NAME --target x86_64-unknown-linux-gnu
      ```

- Use rustfmt and clippy:

  ```
  cargo fmt
  cargo clippy
  ```
