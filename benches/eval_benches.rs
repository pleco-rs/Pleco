#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use pleco::{SQ,BitBoard,Player,Piece,Board};
use pleco::tools::*;
use pleco::board::eval::Eval;

use test::{black_box, Bencher};

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        let mut vec = Vec::new();
        vec.push(Board::default());
        for _x in 0..100 {
            vec.push(gen_rand_no_check());
        }
        vec
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


