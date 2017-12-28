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