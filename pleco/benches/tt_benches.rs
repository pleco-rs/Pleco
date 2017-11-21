#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

use self::pleco::tools::tt::*;
use self::pleco::core::piece_move::BitMove;
use self::test::{black_box, Bencher};
use pleco::tools::prng::PRNG;

#[bench]
fn tt_bench_single_thread_insert_empty(b: &mut Bencher) {
    let tt = TT::new_num_entries(400_000);
    let mut prng = PRNG::init(1120246457);
    b.iter(|| {
        let key = prng.rand();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
    })
}

#[bench]
fn tt_bench_single_thread_insert_full(b: &mut Bencher) {
    let tt = TT::new_num_entries(400_000);
    let mut prng = PRNG::init(2500123475);

    for x in 0..1_600_000 {
        let key = prng.rand();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
        entry.depth = x as u8;
    }

    b.iter(|| {
        let key = prng.rand();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, key as u8, NodeBound::Exact);
    })
}

#[bench]
fn tt_bench_single_thread_lookup_sparse(b: &mut Bencher) {
    let seed: u64 = 7736583456;
    tt_single_thread_lookup(b,200_000, 20_000, seed);
}


#[bench]
fn tt_bench_single_thread_lookup_dense(b: &mut Bencher) {
    let seed: u64 = 80474222;
    tt_single_thread_lookup(b,200_000, 500_000, seed);
}

#[inline]
fn tt_single_thread_lookup(b: &mut Bencher, num_entries: usize, placements: u64, seed: u64) {
    let tt = TT::new_num_entries(num_entries);
    let mut prng = PRNG::init(seed);

    for x in 0..placements {
        let key = prng.rand();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
        entry.depth = x as u8;
    }

    b.iter(|| {
        let key = prng.rand();
        let (_found, _entry) = black_box(tt.probe(key));
    });
}