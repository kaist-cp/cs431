# Parallel web server with cache

## Expected outcome

- Execute `cargo run hello_server`. A web server should run.
- Browse `http://localhost:7878/alice`. It should wait for a few seconds, and returns a web page.
- Browse `http://localhost:7878/alice` again. It should instantly return a web page.
- Browse `http://localhost:7878/bob`. It should wait for a few seconds, and returns a web page.
- Press `Ctrl-C`. The web server should gracefully shut down after printing statistics.

## Organization

- `./src/bin/hello_server.rs`: the web server.
- `./src/hello_server/*.rs`: the server components. You should fill out `todo!()` in those files.

## Guide

- Read [the Rust book](https://doc.rust-lang.org/book/) to the end. This homework is based on the
  book's [final project](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html).
