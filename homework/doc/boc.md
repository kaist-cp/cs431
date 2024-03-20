# Behaviour-Oriented Concurrency (BoC)
**Implement a runtime for Behaviour-Oriented Concurrency**

> *The Behaviour-Oriented Concurrency paradigm: a concurrency paradigm
> that achieves flexible coordination over multiple resources, and ordered execution, and scalability.* (from §1 of the BoC [paper](https://doi.org/10.1145/3622852))

First, read [the original BoC paper](https://doi.org/10.1145/3622852) and understand its algorithm.
In particular, you should understand the key concepts (e.g., cown, behaviour, when, and thunk), and fully understand `Fig.3` and `§4.3` which contain the details of the implementation.

Fill in the `todo!()`s in `src/boc.rs`.
The total lines of code to be written is about 70.
Your implementation should satisfy the following criterias:
* when clauses should be scheduled in the correct order of the *dependency graph* (§4.1).
* Your implementation of the BoC runtime should ensure *deadlock freedom*.
  We will test the deadlock freedom by several stress tests with timeouts.
* Whenever you want to spawn a new thread, **don't use** [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html).
  Instead, use [`rayon::spawn`](https://docs.rs/rayon/latest/rayon/fn.spawn.html).

We provide several ways of using the when clause in Rust, illustrated below.

1.  Using the `when!` macro. Below is a representative example describing its use:

    ```rust
    when!(c1, c2; g1, g2; {
        ... // thunk
    });
    ```
    This results in a when clause that schedules a new behavior for two `CownPtr`s `c1` and `c2`.
    `g1` and `g2` are mutable references to the shared resources protected by given `CownPtr`s
    and can be used in the thunk.
2.  Using the `run_when` function directly. Use this if you want to create a new behavior with an arbitrary number of `CownPtr`s.
    For example,

    ```rust
    run_when(vec![c1.clone(), c2.clone(), c3.clone()], move |mut acc| {
        ... // thunk
    });
    ```
    The first argument is a `Vec` of cowns with the same type.
    `acc` is a vector of mutable references to the shared resources protected by the cowns,
    and it is guaranteed that `acc` has the same length as the specified given `Vec` of `CownPtr`s.

More examples can be found in `src/boc.rs` and `test/boc.rs`.

## Grading (100 points)
Run `./scripts/grade-boc.sh`.
Basic tests account for 60 points and stress tests account for 40 points.

Note: You don't need to worry about the message (shown below) that might be printed during the tests with `cargo_tsan`.
It will not affect the grading.
```
/usr/bin/addr2line: DWARF error: invalid or unhandled FORM value: 0x23
```

## Submission
Submit `boc.rs` to gg.
