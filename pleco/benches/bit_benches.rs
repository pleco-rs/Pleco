#![feature(test)]
extern crate pleco;
extern crate test;

use pleco::core::bit_twiddles::*;

use test::{black_box, Bencher};

#[macro_use]
extern crate lazy_static;

use pleco::core::bitboard::{BitBoard,RandBitBoard};

lazy_static! {
    pub static ref BIT_SETS_DENSE_1000: Vec<BitBoard> = {
        RandBitBoard::default().pseudo_random(2661634).avg(6).max(11).many(1000)
    };
}

#[bench]
fn bench_popcount_1000_rust(b: &mut Bencher) {
    b.iter(|| {
        black_box({
            for bits in BIT_SETS_DENSE_1000.iter() {
                black_box({black_box(black_box((*bits).0)).count_ones();})
            }
        })
    })
}

#[bench]
fn bench_popcount_1000_old(b: &mut Bencher) {
    b.iter(|| {
        black_box({
            for bits in BIT_SETS_DENSE_1000.iter() {
                black_box({popcount_old(black_box((*bits).0));})
            }
        })
    })
}




