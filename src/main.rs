extern crate pleco;

use pleco::uci::console_loop;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    console_loop(args);
}