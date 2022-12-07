use std::io::{self, BufRead};

use strip_ansi::ansi;

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        println!("{}", ansi::strip_ansi(line.as_str()));
    }
}
