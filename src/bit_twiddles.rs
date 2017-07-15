
use test::Bencher;
//use std::mem;
use test;

static POPCNT8: &'static [u8] = &[
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8
];

static DEBRUIJ_T: &'static [u8] = &[
    0, 47,  1, 56, 48, 27,  2, 60,
    57, 49, 41, 37, 28, 16,  3, 61,
    54, 58, 35, 52, 50, 42, 21, 44,
    38, 32, 29, 23, 17, 11,  4, 62,
    46, 55, 26, 59, 40, 36, 15, 53,
    34, 51, 20, 43, 31, 22, 10, 45,
    25, 39, 14, 33, 19, 30,  9, 24,
    13, 18,  8, 12,  7,  6,  5, 63
];

const DEBRUIJ_M: u64 = 0x03f79d71b4cb0a89;

// BitScanForward: Djuin:            9 s
// BitScanForward: PopCount - Old : 18 s
// BitScanForward: PopCount - Rust: 26 s

// PopCount: Rust:  22 s
// PopCount: Old :  37 s




// Returns count of bits
#[inline(always)]
pub fn popcount64(x: u64) -> u8 {
     x.count_ones() as u8
}


//// Pops and Returns the lsb
//#[inline]
//pub fn pop_lsb(x: &mut u64) -> u64 {
//    let lsb: Bitboard = lsb(*x);
//    x &= !lsb;
//    lsb
//}

// Returns index of the LSB
#[inline(always)]
pub fn bit_scan_forward(bits: u64) -> u8 {
    assert_ne!(bits, 0);
    DEBRUIJ_T[(((bits ^ bits.wrapping_sub(1)).wrapping_mul(DEBRUIJ_M)).wrapping_shr(58)) as usize]
}

#[inline(always)]
pub fn bit_scan_forward_rust_trailing(bits: u64) -> u8 {
    assert_ne!(bits, 0);
    bits.trailing_zeros() as u8
}

#[inline(always)]
pub fn bit_scan_reverse(mut bb: u64) -> u8 {
    assert_ne!(bb, 0);
    bb |= bb >> 1;
    bb |= bb >> 2;
    bb |= bb >> 4;
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    DEBRUIJ_T[(bb.wrapping_mul(DEBRUIJ_M)).wrapping_shr(58) as usize]
}
#[inline(always)]
pub fn more_than_one(x: u64) -> bool {
    (x & (x.wrapping_sub(1))) != 0
}

// Returns the LSB
#[inline(always)]
pub fn lsb(bits: u64) -> u64 {
    (1 as u64).wrapping_shl(bits.trailing_zeros())
}

#[inline(always)]
pub fn msb(bits: u64) -> u64 {
    (1 as u64).wrapping_shl(bits.leading_zeros())
}

#[inline(always)]
fn popcount_old(x: u64) -> u8 {
    let x = x as usize;
    if x == 0 { return 0 }
    if x & (x.wrapping_sub(1)) == 0 { return 1 }
    POPCNT8[x >> 56] +
        POPCNT8[(x >> 48) & 0xFF] +
        POPCNT8[(x >> 40) & 0xFF] +
        POPCNT8[(x >> 32) & 0xFF] +
        POPCNT8[(x >> 24) & 0xFF] +
        POPCNT8[(x >> 16) & 0xFF] +
        POPCNT8[(x >> 8) & 0xFF] +
        POPCNT8[x & 0xFF]
}




pub const TRAILS: u64 = 17000;


#[bench]
fn evs_bench_bitscan_djuie(b: &mut Bencher) {
    b.iter(|| {
        let n: u64 = test::black_box(TRAILS);
        (0..n).fold(0, |a, c| {
            let mut x: u64 = very_sparse_random(c.wrapping_mul(909090909090909091));
            if x == 0 { x = 1;} else { x = bit_scan_forward(x) as u64;}
            a ^ (x) }
        )
    })
}



#[bench]
fn evs_bench_popcount_rust(b: &mut Bencher) {
    b.iter(|| {
        let n: u64 = test::black_box(TRAILS);
        (0..n).fold(0, |a, c| {
            let mut x: u64 = very_sparse_random(c.wrapping_mul(909090909090909091));
            if x == 0 { x = 1;} else { x = popcount64(x) as u64;}
            a ^ (x) }
        )
    })
}

#[bench]
fn evs_bench_popcount_old(b: &mut Bencher) {
    b.iter(|| {
        let n: u64 = test::black_box(TRAILS);
        (0..n).fold(0, |a, c| {
            let mut x: u64 = very_sparse_random(c.wrapping_mul(909090909090909091));
            if x == 0 { x = 1;} else { x = popcount_old(x) as u64;}
            a ^ (x) }
        )
    })
}


#[bench]
fn evs_bench_lsb_pop_rust(b: &mut Bencher) {
    b.iter(|| {
        let n: u64 = test::black_box(TRAILS);
        (0..n).fold(0, |a, c| {
            let mut x: u64 = very_sparse_random(c.wrapping_mul(909090909090909091));
            if x == 0 { x = 1;} else { x = lsb(x) as u64;}
            a ^ (x)
        })
    })
}


#[bench]
fn evs_bench_randomize_super_sparse(b: &mut Bencher) {
    b.iter(|| {
        let n: u64 = test::black_box(TRAILS);
        (0..n).fold(0, |a, c| {
            let mut x: u64 = very_sparse_random(c.wrapping_mul(909090909090909091));
            if x == 0 { x = 1;}
            a ^ (x) }
        )
    })
}



#[inline]
fn randomize(x: u64) -> u64{
    (!random_2(random_1(x))  ^ random_4(!x)) & random_1(x)
}

// Densest
#[inline]
fn randomize2(x: u64) -> u64{
    random_1(!random_2(x))  ^ !random_3(!x) ^ random_4(x)
}

#[inline]
fn randomize_sparse (x: u64) -> u64{
    (!random_3(random_2(!x))) & random_1(x) & random_4(x)
}

#[inline]
fn very_sparse_random (x: u64) -> u64{
    ((!8512677386048191063) ^ !(x + 6)).wrapping_mul(!x) & ((!1030501117050341) ^ x).wrapping_mul(!x) & (! 1030507050301 as u64).wrapping_mul(!x) & random_2(x) & (!x).wrapping_mul(2685821657736338717) & !random_4(x) & random_3(!x) & (x).wrapping_mul(268582165773633871)
}




fn random_3(x: u64) -> u64 {
    let mut c: u64 = x.wrapping_shr(12);
    c ^= x.wrapping_shl(25);
    c ^= x.wrapping_shr(27);
    x.wrapping_mul(1030507050301) ^ c
}

fn random_2(x: u64) -> u64 {
    let mut c: u64 = x.wrapping_shr(12);
    c ^= x.wrapping_shl(25);
    c ^= x.wrapping_shr(27);
    x.wrapping_mul(8512677386048191063) ^ c
}

fn random_1(x: u64) -> u64 {
    let mut c: u64 = x.wrapping_shr(12);
    c ^= x.wrapping_shl(25);
    c ^= x.wrapping_shr(27);
    x.wrapping_mul(2685821657736338717) ^ c
}

fn random_4(x: u64) -> u64 {
    let mut c: u64 = x.wrapping_shr(12);
    c ^= x.wrapping_shl(25);
    c ^= x.wrapping_shr(27);
    x.wrapping_mul(399899999999999 ) ^ c
}
