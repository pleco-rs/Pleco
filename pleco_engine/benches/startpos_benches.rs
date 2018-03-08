#![feature(test)]
extern crate test;
extern crate pleco;
extern crate pleco_engine;

use pleco::Board;

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;


use test::{black_box, Bencher};

#[bench]
fn engine_3_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(3);
    let board = Board::default();
    let mut s = black_box(PlecoSearcher::init(false));
    b.iter(|| {
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}



#[bench]
fn engine_4_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(4);
    let board = Board::default();
    let mut s = black_box(PlecoSearcher::init(false));
    b.iter(|| {
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}


#[bench]
fn engine_5_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(5);
    let board = Board::default();
    let mut s = black_box(PlecoSearcher::init(false));
    b.iter(|| {
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

#[bench]
fn engine_6_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(6);
    let board = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}


#[bench]
fn engine_7_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(7);
    let board = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}