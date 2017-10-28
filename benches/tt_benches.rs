#![feature(test)]
extern crate pleco;
extern crate test;
extern crate rand;

use self::pleco::tt::*;
use self::pleco::core::piece_move::BitMove;
use self::test::{black_box, Bencher};


#[bench]
fn tt_bench_single_thread_insert_empty(b: &mut Bencher) {
    let tt = TT::new_num_entries(400_000);
    b.iter(|| {
        let key = rand::random::<u64>();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
    })
}

#[bench]
fn tt_bench_single_thread_insert_full(b: &mut Bencher) {
    let tt = TT::new_num_entries(400_000);

    for x in 0..1_600_000 {
        let key = rand::random::<u64>();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
        entry.depth = x as u8;
    }

    b.iter(|| {
        let key = rand::random::<u64>();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, key as u8, NodeBound::Exact);
    })
}

#[bench]
fn tt_bench_single_thread_lookup_sparse(b: &mut Bencher) {
    tt_single_thread_lookup(b,200_000, 20_000);
}


#[bench]
fn tt_bench_single_thread_lookup_dense(b: &mut Bencher) {
    tt_single_thread_lookup(b,200_000, 500_000);
}

#[inline]
fn tt_single_thread_lookup(b: &mut Bencher, num_entries: usize, placements: u64) {
    let tt = TT::new_num_entries(num_entries);

    for x in 0..placements {
        let key = rand::random::<u64>();
        let (_found, entry) = tt.probe(key);
        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact);
        entry.depth = x as u8;
    }

    b.iter(|| {
        let key = rand::random::<u64>();
        let (_found, _entry) = black_box(tt.probe(key));
    });
}