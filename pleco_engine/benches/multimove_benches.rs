#![feature(test)]
extern crate test;
extern crate pleco;
extern crate pleco_engine;

use pleco::{Board};

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;


use test::{black_box, Bencher};


#[bench]
fn multi_3_engine_4_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(4);
    let startpos = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        let mut board = startpos.clone();
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

#[bench]
fn multi_3_engine_5_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(5);
    let startpos = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        let mut board = startpos.clone();
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

#[bench]
fn multi_3_engine_6_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(6);
    let startpos = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        let mut board = startpos.clone();
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

#[bench]
fn multi_3_engine_7_ply(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(7);
    let startpos = Board::default();
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        let mut board = startpos.clone();
        black_box(s.clear_search());
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}