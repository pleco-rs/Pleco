#![feature(test)]
extern crate pleco;
extern crate test;
extern crate pleco_engine;

use pleco::Board;
use pleco::engine::UCILimit;
use pleco_engine::pleco_searcher::PlecoSearcher;


use test::{black_box, Bencher};




#[bench]
fn bench_4_ply(b: &mut Bencher) {
    let limit = UCILimit::Depth(4);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search());
    })
}


#[bench]
fn bench_5_ply(b: &mut Bencher) {
    let limit = UCILimit::Depth(5);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search());
    })
}

#[bench]
fn bench_6_ply(b: &mut Bencher) {
    let limit = UCILimit::Depth(6);
    let board = Board::default();
    b.iter(|| {
        let mut s = PlecoSearcher::init(false);
        black_box(s.search(&board, &limit));
        black_box(s.stop_search());
    })
}
