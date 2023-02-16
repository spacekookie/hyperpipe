use hyperpipe::HyperPipe;
use std::{
    io::{self, Write},
    path::Path,
    thread,
};

fn main() {
    let mut pipe = HyperPipe::new(Path::new("./test-pipe-data"), 0).unwrap();

    eprintln!("Press Ctrl-C to exit this pull loop!");
    loop {
        match pipe.pull() {
            Some(data) => io::stdout().write_all(data.as_slice()).unwrap(),
            None => {
                // Make a duration for 200ms
                let dur = std::time::Duration::from_millis(200);
                thread::sleep(dur) //thread::yield_now(),
            }
        }
    }
}
