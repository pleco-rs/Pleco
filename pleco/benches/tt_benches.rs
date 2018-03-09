//#![feature(test)]
//extern crate pleco;
//extern crate test;
//extern crate rand;
//
//#[macro_use]
//extern crate criterion;
//use criterion::{Criterion,black_box};
//
//use self::pleco::tools::tt::*;
//use self::pleco::core::piece_move::BitMove;
//
//use pleco::tools::prng::PRNG;
//
//#[bench]
//fn tt_bench_single_thread_insert_empty(c: &mut Criterion) {
//    let tt = TranspositionTable::new_num_entries(400_000);
//    let mut prng = PRNG::init(1120246457);
//    c.bench_function(" tt_bench_single_thread_insert_empty", |b|
//    b.iter(|| {
//        let key = prng.rand();
//        let (_found, entry) = tt.probe(key);
//        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact, tt.time_age());
//    })
//    );
//}
//
//#[bench]
//fn tt_bench_single_thread_insert_full(c: &mut Criterion) {
//    let tt = TranspositionTable::new_num_entries(400_000);
//    let mut prng = PRNG::init(2500123475);
//
//    for x in 0..1_600_000 {
//        let key = prng.rand();
//        let (_found, entry) = tt.probe(key);
//        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact, tt.time_age());
//        entry.depth = x as i8;
//    }
//
//    c.bench_function(" bench_popcount_1000_rust", |b|
//        b.iter(|| {
//    b.iter(|| {
//        let key = prng.rand();
//        let (_found, entry) = tt.probe(key);
//        entry.place(key, BitMove::new(0x555), 3, 4, key as i16, NodeBound::Exact, tt.time_age());
//    })
//}
//
//#[bench]
//fn tt_bench_single_thread_lookup_sparse(c: &mut Criterion) {
//    let seed: u64 = 7736583456;
//
//    tt_single_thread_lookup(b,200_000, 20_000, seed);
//}
//
//
//#[bench]
//fn tt_bench_single_thread_lookup_dense(c: &mut Criterion) {
//    let seed: u64 = 80474222;
//    tt_single_thread_lookup(b,200_000, 500_000, seed);
//}
//
//#[inline]
//fn tt_single_thread_lookup(b: &mut Bencher, num_entries: usize, placements: u64, seed: u64) {
//    let tt = TranspositionTable::new_num_entries(num_entries);
//    let mut prng = PRNG::init(seed);
//
//    for x in 0..placements {
//        let key = prng.rand();
//        let (_found, entry) = tt.probe(key);
//        entry.place(key, BitMove::new(0x555), 3, 4, 3, NodeBound::Exact, tt.time_age());
//        entry.depth = x as i8;
//    }
//
//    b.iter(|| {
//        let key = prng.rand();
//        let (_found, _entry) = black_box(tt.probe(key));
//    });
//}