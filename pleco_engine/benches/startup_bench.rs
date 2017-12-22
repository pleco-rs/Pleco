#![feature(test)]
extern crate pleco;
extern crate test;
extern crate pleco_engine;

use pleco::Board;

use pleco_engine::pleco_searcher::PlecoSearcher;
use pleco_engine::pleco_searcher::misc::PreLimits;


use test::{black_box, Bencher};

#[bench]
fn searcher_creation(b: &mut Bencher) {
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        s = black_box(PlecoSearcher::init(false));
    })
}

#[bench]
fn searcher_setup(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(0);
    let board = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        black_box(s.clear_tt());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}