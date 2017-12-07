#![feature(test)]
extern crate pleco;
extern crate test;
extern crate pleco_engine;

use pleco::Board;

use pleco_engine::pleco_searcher::PlecoSearcher;
use pleco_engine::pleco_searcher::misc::PreLimits;


use test::{black_box, Bencher};




#[bench]
fn bench_4_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(4);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search_get_move());
    })
}


#[bench]
fn bench_5_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(5);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search_get_move());
    })
}

#[bench]
fn bench_6_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(6);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search_get_move());
    })
}