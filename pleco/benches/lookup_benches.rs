use std::time::Duration;

use criterion::{black_box, Bencher, Criterion};

use pleco::helper::prelude::*;
use pleco::{BitBoard, SQ};

fn lookup_tables(c: &mut Criterion) {
    init_statics();

    c.bench_function("king_lookup", king_lookup);
    c.bench_function("knight_lookup", knight_lookup);
    c.bench_function("bishop_lookup", bishop_lookup);
    c.bench_function("rook_lookup", rook_lookup);
    c.bench_function("queen_lookup", queen_lookup);
    c.bench_function("multi_lookup_sequential", multi_lookup_sequential);
    c.bench_function("multi_lookup_stutter", multi_lookup_stutter);
}

fn king_lookup(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let x: u64 = black_box(knight_moves(SQ(c)).0);
            a ^ (x)
        })
    })
}

fn knight_lookup(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let x: u64 = black_box(king_moves(SQ(c)).0);
            a ^ (x)
        })
    })
}

fn rook_lookup(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let x: u64 = black_box(rook_moves(BitBoard(a), SQ(c)).0);
            a ^ (x)
        })
    })
}

fn bishop_lookup(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let x: u64 = black_box(bishop_moves(BitBoard(a), SQ(c)).0);
            a ^ (x)
        })
    })
}

fn queen_lookup(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let x: u64 = black_box(queen_moves(BitBoard(a), SQ(c)).0);
            a ^ (x)
        })
    })
}

// Benefits from locality
fn multi_lookup_sequential(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let mut x: u64 = black_box(knight_moves(SQ(c)).0);
            x ^= king_moves(SQ(c)).0;
            x ^= bishop_moves(BitBoard(x), SQ(c)).0;
            x ^= rook_moves(BitBoard(x), SQ(c)).0;
            x ^= black_box(queen_moves(BitBoard(x), SQ(c)).0);
            a ^ (x)
        })
    })
}

// Stutters so Cache must be refreshed more often
fn multi_lookup_stutter(b: &mut Bencher) {
    b.iter(|| {
        (0..64).fold(0, |a: u64, c| {
            let mut x: u64 = queen_moves(BitBoard(a), SQ(c)).0;
            x ^= king_moves(SQ(c)).0;
            x ^= bishop_moves(BitBoard(a), SQ(c)).0;
            x ^= knight_moves(SQ(c)).0;
            x ^= black_box(rook_moves(BitBoard(x), SQ(c)).0);
            a ^ (x)
        })
    })
}

criterion_group!(name = lookup_benches;
     config = Criterion::default()
        .sample_size(250)
        .warm_up_time(Duration::from_millis(3));
    targets = lookup_tables
);
