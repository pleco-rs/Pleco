#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use pleco::board::perft::*;
use pleco::board::Board;

use test::{black_box, Bencher};

lazy_static! {
    pub static ref RAND_BOARDS: Vec<Board> = {
        RAND_BOARDS_ALL.iter()
            .map(|b| Board::new_from_fen(b).unwrap())
            .collect::<Vec<Board>>()
    };
}

#[bench]
fn perft_3(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS.iter() {
            black_box(perft(board, 3));
        }
    })
}

#[bench]
fn perft_4(b: &mut Bencher) {
    b.iter(|| {
        for board in RAND_BOARDS.iter() {
            black_box(perft(board, 4));
        }
    })
}


static RAND_BOARDS_ALL: [&str; 6] = [
    "rn2k3/pp1qPppr/5n2/1b2B3/8/4NP2/3NP1PP/R2K1B1R b q - 0 23",
    "r1bqkbnr/ppp2ppp/2np4/4p3/4PQ2/2NP4/PPP1NPPP/R1B1KB1R w KQkq e6 0 8",
    "r1bqkb1r/pp2pp2/2p2n2/6Q1/7p/2N4P/PP1B1PP1/R3KBNR w KQkq - 0 14",
    "3k4/6b1/1p5p/4p3/5rP1/6K1/8/ w - - 0 40",
    "1k6/1p1n4/p6p/4P3/2P5/1R6/5K1P/4R b - - 2 33",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"];