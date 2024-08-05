# Parallel web server with cache

## Expected outcome

- Execute `cargo run --features="build-bin" hello_server`. A web server should run. If it doesn't, try changing the port used in [`hello_server.rs:6`](../src/bin/hello_server.rs).
- Run `curl http://localhost:7878/alice`. It should wait for a few seconds, and return a web page.
- Run `curl http://localhost:7878/alice` again. It should instantly return a web page.
- Run `curl http://localhost:7878/bob`. It should wait for a few seconds, and return a web page.
- Press `Ctrl-C`. The web server should gracefully shut down after printing statistics.

## Organization

- `../src/bin/hello_server.rs`: the web server.
- `../src/hello_server/*.rs`: the server components. You should fill out `todo!()` in those files.

## Grading
The grader runs `./scripts/grade-hello_server.sh` in the `homework` directory.
This script runs the tests with various options.

There will be no partial scores for `tcp` and `thread_pool` modules.
That is, you will get the score for a module only if your implementation passes **all** tests for that module.

On the other hand, we will give partial scores for `cache` module.
In particular, even if your implementation of `cache` blocks concurrent accesses to different keys, you can still get some points for basic functionalities.

## Submission
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-hello_server.zip
```
Submit `hw-hello_server.zip` to gg.

## Guide

### Reading Rust book
This homework requires a good understanding of the materials covered in [the Rust book §20](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html).
This is the minimal path for understanding §20: §1, 2, 3, 4, 5, 6, 8, 9, 10, 13.1, 13.2, **15**, **16**, **20**.

Specifically, make sure that you understand the following topics.
* [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) trait and [`drop`](https://doc.rust-lang.org/std/mem/fn.drop.html) function
* Type signature of [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html) and the meaning of [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html).
* The meaning and usage of [`Arc<`](https://doc.rust-lang.org/std/sync/struct.Arc.html)[`Mutex<T>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
* [Channels](https://doc.rust-lang.org/std/sync/mpsc/index.html).
<!-- * The fact that there is no non-trivial way to break out of `TcpListener::incoming` loop. -->

See also: Rust book with quiz. <https://rust-book.cs.brown.edu/>

### Major differences between HW1 thread pool and Rust book §20 thread pool
1. We use [`crossbeam_channel`](https://docs.rs/crossbeam-channel/) instead of [<code>std::sync::<strong>mpsc</strong></code>](https://doc.rust-lang.org/std/sync/mpsc/index.html). Since crossbeam's channels are **mpmc**, you don't need to wrap the `Receiver` inside a `Mutex`.
1. We do not use explicit exit messages for the thread pool. Instead, we disconnect the channel by `drop`ping the receiver/sender.
    * Our message type is simply the `Job` itself:
      ```rust
      struct Job(Box<dyn FnOnce() + Send + 'static>);
      ```
    * Each worker thread automatically breaks out of the loop if the channel is disconnected.
1. We `join()` each thread in the destructor of `Worker`, not in the destructor of `ThreadPool`. Since `ThreadPool` has field `workers: Vec<Worker>`, the worker destructor will be called when the pool is dropped. Note that the channel should be disconnected before `join()`ning the worker threads. (Otherwise, `join` will block.) This means that the `Sender` should be dropped before `Vec<Worker>`. You can specify the drop order in many ways. In this homework, we use `ThreadPool::job_sender` of type `Option<Sender<Job>>`, whose content can be `take()`n and `drop()`ped explicitly in `<ThreadPool as Drop>::drop`.

### Tips
* Cache: Start with `Mutex<HashMap<K, V>>`. To fully implement the specification, you will need a more complicated type. The simplest solution makes use of all the things imported in `cache.rs`.
* Interrupt handler: just follow the comments.
* Thread pool: Ignore `ThreadPoolInner` first (it's used for `ThreadPool::join`), and implement the changes discussed above.
* If you have questions, try looking up the [issue tracker](https://github.com/kaist-cp/cs431/issues).
  There are many Q&A's from the previous iterations of this course, and they are labeled by the topic.
  For example, ["homework - cache" label](https://github.com/kaist-cp/cs431/issues?q=label%3A%22homework+-+cache%22+) lists the questions about `cache.rs`.
  Here are some Q&A's you may find useful for this homework:
    * https://github.com/kaist-cp/cs431/issues/339
    * https://github.com/kaist-cp/cs431/issues/85#issuecomment-696888546
    * https://github.com/kaist-cp/cs431/issues/81

### Testing
We'll only test the libraries.
```bash
cargo test --test cache
cargo test --test tcp
cargo test --test thread_pool
```
We will use those tests for grading, too. We may add some more tests for grading, but if your solution passes all the given tests, you will likely get the full score.

Also, try running tests with the [LLVM sanitizers](https://github.com/kaist-cp/cs431/tree/main/homework#using-llvm-sanitizers) enabled.
They are not that useful for HW1, but they will be very helpful for upcoming homework assignments.
