# Hyper Pipe

A FIFO (first-in-first-out) pipe, using the file system to store
buffered values.  **This is an educational crate and should not be
used for moving important data!**  Importantly this crate MAY produce
a data races, since it uses file writes instead of a write and rename,
which is atomic.  It also can only handle a single producer and a
single consumer!

This crate was written as part of the advanced Async course in
[Learning Rust](https://git.irde.st/kookiespace/learning-rust).

If you end up using this crate in your own code, may god have mercy on
your soul.  Also, please send patches for anything you improve.
