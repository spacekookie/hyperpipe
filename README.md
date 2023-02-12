# Hyper Pipe

A FIFO (first-in-first-out) pipe, using the file system to store
buffered values.  **This is an educational crate and should not be
used for moving important data!**  Importantly this crate MAY produce
a data race between the writing and reading side of the pipe.  It also
can only handle a single producer and a single consumer!

This crate was written as part of the advanced Async course in
[learning-rust](https://github.com/spacekookie/learning-rust).
