# Hyper Pipe

A FIFO (first-in-first-out) pipe, using the file system to store
buffered values.  **This is an educational crate and should not be
used for moving important data!**  While atomic write operations are
used to update the pipe manifest, a HypePipe only supports **a single
producer and a single consumer**.

This crate was written as part of the advanced Async course in
[Learning Rust](https://git.irde.st/kookiespace/learning-rust).

If you end up using this crate in your own code, may god have mercy on
your soul.  Also, please send patches for anything you improve.


## How to use

Check out the `src/bin` directory for two short examples.

```rust
fn main() {
    let pipe_path = Path::new("buffer-dir");

    let mut p1 = HyperPipe::new(pipe_path).unwrap();
    let v1 = vec![1, 2, 3, 4, 5, 6];
    p1.push(v1.clone()).unwrap();

    let mut p2 = HyperPipe::new(pipe_path).unwrap();
    let v2 = p2.pull().unwrap();
    assert_eq!(v1, v2);
}
```

A `HypePath` type MUST only either be used to push or pull.  The
producer and consumer must hold different instances to the same pipe,
even inside the same program.

Additionally no concurrency-guarantees can made for having multiple
producers or multiple consumers.


## License

Hyperpipe is free software licensed under the GNU General Public License
3.0 or later.  A copy of this license is included in the repository.
