#![feature(test)]

extern crate pleco;
extern crate test;

use pleco::helper::Helper;
use pleco::{SQ,BitBoard};
use test::{black_box, Bencher};



#[bench]
fn bench_rook_lookup(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.rook_moves(BitBoard(a),SQ(c)).0;
            a ^ (x) }
        )
    })
}


#[bench]
fn bench_bishop_lookup(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.bishop_moves(BitBoard(a),SQ(c)).0;
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_queen_lookup(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.queen_moves(BitBoard(a),SQ(c)).0;
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_king_lookup(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.king_moves(SQ(c)).0;
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_knight_lookup(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.knight_moves(SQ(c)).0;
            a ^ (x) }
        )
    })
}

// Benefits from locality
#[bench]
fn bench_multi_lookup_sequential(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.knight_moves(SQ(c)).0;
            x ^= m.king_moves(SQ(c)).0;
            x ^= m.bishop_moves(BitBoard(x),SQ(c)).0;
            x ^= m.rook_moves(BitBoard(x),SQ(c)).0;
            x ^= m.queen_moves(BitBoard(x),SQ(c)).0;
            a ^ (x) }
        )
    })
}


// Stutters so Cache must be refreshed more often
#[bench]
fn bench_multi_lookup_stutter(b: &mut Bencher) {
    let m = Helper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.queen_moves(BitBoard(a),SQ(c)).0;
            x ^= m.king_moves(SQ(c)).0;
            x ^= m.bishop_moves(BitBoard(a),SQ(c)).0;
            x ^= m.knight_moves(SQ(c)).0;
            x ^= m.rook_moves(BitBoard(a),SQ(c)).0;
            a ^ (x) }
        )
    })
}
