# Doubly linked list
**Implement doubly linked list in unsafe Rust.**

This homework serves as a brief tutorial for unsafe Rust with a focus on the basic raw pointer operations.

The [skeleton code](https://github.com/kaist-cp/cs431/blob/main/homework/src/linked_list.rs) is slightly modified version of [the linked list from Rust standard library](https://doc.rust-lang.org/std/collections/struct.LinkedList.html).
We already provided implementation for several methods e.g. `push_front_node`.
Your job is to implement their symmetric counterparts
e.g. `push_back_node` and some methods of `IterMut` struct (see `todo!()`s).

You can look up its implementation from the standard library,
but we encourage you do it yourself
so that you can build enough skill set for upcoming homeworks.
We also recommend you to play around with AddressSanitizer and debugger.

## Grading
* The full score for this homework is 40 points (HW1 was 100) and the total lines of code to be written is about 80.
* You can evaluate your solution by running `./scripts/grade-linked_list.sh` in the `homework` directory.

## Submission
Submit `linked_list.rs` to gg.

## Guide

### Learn the basics of unsafe Rust
1. Read [Rust Book §19.1](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
1. Skim through [Nomicon §1](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
1. Read [raw pointer type documentation](https://doc.rust-lang.org/std/primitive.pointer.html) and some of its methods (`is_null`, `as_ref`, `read`, `write`, `replace`, `swap`)
1. Read [`std::mem`](https://doc.rust-lang.org/std/mem/index.html) and [`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html) documentations.
1. Read [`std::iter`](https://doc.rust-lang.org/std/iter/index.html) documentation.

### Tips for debugging
When `cargo test` fails with error messages like this,
```
thread panicked while panicking. aborting.
error: test failed, to rerun pass '--test linked_list'

Caused by:
  process didn't exit successfully: ... (signal: 4, SIGILL: illegal instruction)
```
try running the test like this
<pre>
cargo test --test linked_list <strong>-- --nocapture --test-threads 1</strong>
</pre>
This will give you more informative error messages.

### Other useful resources
* [`*` operator](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#the-dereference-operator)
* [`.` operator](https://doc.rust-lang.org/stable/reference/expressions/call-expr.html)
* [type coercion (weakening)](https://doc.rust-lang.org/nomicon/coercions.html)
* [type casting](https://doc.rust-lang.org/nomicon/casts.html)
* [`Box<T>` is special](https://doc.rust-lang.org/stable/reference/special-types-and-traits.html#boxt)
* [`*const T` vs. `*mut T`](https://internals.rust-lang.org/t/what-is-the-real-difference-between-const-t-and-mut-t-raw-pointers/6127)
