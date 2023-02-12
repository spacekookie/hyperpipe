use hyperpipe::HyperPipe;
use std::{
    io::{self, Read},
    path::Path,
};

fn main() {
    let mut pipe = HyperPipe::new(Path::new("./test-pipe-data")).unwrap();

    let mut input = vec![];
    io::stdin().read_to_end(&mut input).unwrap();

    let r = pipe.push(input);
    eprintln!(
        "Push element: {}",
        if r.is_some() { "OK" } else { "FAILED" }
    );
}
