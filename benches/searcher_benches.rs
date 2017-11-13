#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

use pleco::Board;
use pleco::pleco_searcher::lazy_smp::*;
use pleco::engine::{Searcher,UCILimit};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;


use test::{black_box, Bencher};

//#[bench]
fn uci_setup(b: &mut Bencher) {
    let board = Board::default();
    let stop = Arc::new(AtomicBool::new(false));
    b.iter(|| {
        black_box(
            PlecoSearcher::setup(board.shallow_clone(),Arc::clone(&stop))
        );

    })
}

//#[bench]
fn uci_setup_and_go(b: &mut Bencher) {
    let board = Board::default();
    let stop = Arc::new(AtomicBool::new(false));
    b.iter(|| {
        black_box({
            let mut m = black_box(PlecoSearcher::setup(board.shallow_clone(), Arc::clone(&stop)));
            black_box(m.uci_go(UCILimit::Depth(0),false));
        });

    })
}