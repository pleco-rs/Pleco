#![feature(test)]
extern crate pleco;
extern crate test;
extern crate pleco_engine;


use pleco_engine::searcher::PlecoSearcher;


use test::{black_box, Bencher};

#[bench]
fn searcher_creation(b: &mut Bencher) {
    let mut s = PlecoSearcher::init(false);
    b.iter(|| {
        s = black_box(PlecoSearcher::init(false));
    })
}