use hyperpipe::HyperPipe;
use std::{
    io::{self, Write},
    path::Path,
    thread,
};

fn main() {
    let mut pipe = HyperPipe::new(Path::new("./test-pipe-data")).unwrap();

    eprintln!("Press Ctrl-C to exit this pull loop!");
    loop {
        match pipe.pull() {
            Some(data) => io::stdout().write_all(data.as_slice()).unwrap(),
            None => thread::sleep_ms(200), //thread::yield_now(),
        }
    }
}
