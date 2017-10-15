#![feature(test)]

extern crate pleco;
extern crate test;

use self::pleco::magic_helper::MagicHelper;
use self::test::{black_box, Bencher};


#[bench]
fn bench_rook_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.rook_moves(a,c);
            a ^ (x) }
        )
    })
}


#[bench]
fn bench_bishop_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.bishop_moves(a,c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_queen_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.queen_moves(a,c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_king_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.king_moves(c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_knight_lookup(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let x: u64 = m.knight_moves(c);
            a ^ (x) }
        )
    })
}

// Benefits from locality
#[bench]
fn bench_multi_lookup_sequential(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.knight_moves(c);
            x ^= m.king_moves(c);
            x ^= m.bishop_moves(x,c);
            x ^= m.rook_moves(x,c);
            x ^= m.queen_moves(x,c);
            a ^ (x) }
        )
    })
}


// Stutters so Cache must be refreshed more often
#[bench]
fn bench_multi_lookup_stutter(b: &mut Bencher) {
    let m = MagicHelper::new();
    b.iter(|| {
        let n: u8 = test::black_box(64);
        (0..n).fold(0, |a: u64, c| {
            let mut x: u64 = m.queen_moves(a,c);
            x ^= m.king_moves(c);
            x ^= m.bishop_moves(x,c);
            x ^= m.knight_moves(c);
            x ^= m.rook_moves(x,c);
            a ^ (x) }
        )
    })
}

#[bench]
fn bench_magic_helper_creation(b: &mut Bencher) {
    b.iter(|| {
        let n: u8 = black_box(1);
        (0..n).fold(0, |a: u64, c| {
            let m = MagicHelper::new();
            let x: u64 = m.king_moves(c);
            a ^ (x) }
        )
    })
}