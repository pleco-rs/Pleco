#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use pleco::board::eval::Eval;
use pleco::board::{RandBoard,Board};

use test::{black_box, Bencher};

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        RandBoard::default().no_check().pseudo_random(730315678).many(100)
    };
}


#[bench]
fn bench_100_evaluations(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS.iter() {
            black_box(Eval::eval_low(board));
        }
    })
}


