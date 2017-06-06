use bit_twiddles::*;
use board::*;
use templates::*;
use std::ptr;
use std::mem;
use std::slice;
use std;
use std::ops::IndexMut;
use std::ops::Range;
use std::ops::Index;
use std::borrow::Borrow;
use std::num::Wrapping;
use std::num;
use std::cmp;

const NIL: u64 = 1;



struct MagicHelper {
    knight_table: [u64; 64],
    king_table: [u64; 64]
}

//impl MagicHelper {
//    pub fn new() -> MagicHelper {
//        MagicHelper {
//            magic_bishop_moves: MagicHelper::gen_magic_bishop(),
//            magic_rook_moves: MagicHelper::gen_magic_rook()
//        }
//    }
//
//    pub fn default() -> MagicHelper { MagicHelper::new() }
//
//    fn gen_magic_bishop() -> [[u64; 4096]; 64] {
//        let mut arr: [[u64; 4096]; 64] = [[0; 4096]; 64];
//        let mut mask: u64 = 0;
//        for bitRef in 0..64 {
//            mask = BISHOP_MASK[bitRef];
//
//        }
//
//    }
//
//    fn gen_magic_rook() -> [[u64; 1024]; 64] {
//        let mut arr: [[u64; 1024]; 64] = [[0; 1024]; 64];
//
//    }
//}


fn gen_king_moves() -> [u64; 64] {
    let mut moves: [u64;64] = [0; 64];

    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;
        // LEFT
        if file != 0 {
            mask |= 1 << (index - 1);
        }
        // RIGHT
        if file != 7 {
            mask |= 1 << (index + 1);
        }
        // UP
        if index < 56  {
            mask |= 1 << (index + 8);
        }
        // DOWN
        if index > 7  {
            mask |= 1 << (index - 8);
        }
        // LEFT UP
        if file != 0 && index < 56 {
            mask |= 1 << (index + 7);
        }
        // LEFT DOWN
        if file != 0 && index > 7 {
            mask |= 1 << (index - 9);
        }
        // RIGHT DOWN
        if file!= 7 && index > 7 {
            mask |= 1 << (index - 7);
        }
        // RIGHT UP
        if file != 7 && index < 56 {
            mask |= 1 << (index + 0);
        }
        moves[index] = mask;
    }
    moves
}

fn gen_knight_moves() -> [u64; 64] {
    let mut moves: [u64;64] = [0; 64];
    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;

        // 1 UP   + 2 LEFT
        if file > 1 && index < 56 {
            mask |= 1 << (index + 6);
        }
        // 2 UP   + 1 LEFT
        if file != 0 && index < 48 {
            mask |= 1 << (index + 15);
        }
        // 2 UP   + 1 RIGHT
        if file != 7 && index < 48 {
            mask |= 1 << (index + 17);
        }
        // 1 UP   + 2 RIGHT
        if file < 6 && index < 56 {
            mask |= 1 << (index + 10);
        }
        // 1 DOWN   + 2 RIGHT
        if file < 6 && index > 7 {
            mask |= 1 << (index - 6);
        }
        // 2 DOWN   + 1 RIGHT
        if file != 7 && index > 15 {
            mask |= 1 << (index - 15 );
        }
        // 2 DOWN   + 1 LEFT
        if file != 0 && index > 15 {
            mask |= 1 << (index - 17 );
        }
        // 1 DOWN   + 2 LEFT
        if file > 1 && index > 7 {
            mask |= 1 << (index - 10 );
        }
        moves[index] = mask;
    }
    moves
}

//Bitboard  RookMasks  [SQUARE_NB];
//Bitboard  RookMagics [SQUARE_NB];
//Bitboard* RookAttacks[SQUARE_NB];
//unsigned  RookShifts [SQUARE_NB];
//
//Bitboard  BishopMasks  [SQUARE_NB];
//Bitboard  BishopMagics [SQUARE_NB];
//Bitboard* BishopAttacks[SQUARE_NB];
//unsigned  BishopShifts [SQUARE_NB];
//Bitboard RookTable[0x19000];  // To store rook attacks
//Bitboard BishopTable[0x1480]; // To store bishop attacks

// RookTable
//fn get_magics() {
//    let mut rook_table: [u64; 0x19000] = [0; 0x19000];
//    let mut bishop_table: [u64; 0x1480] = [0; 0x1480];
//
//    let mut rook_masks: [u64; 64] = [0; 64];
//    let mut rook_magics: [[u64]; 64] = [0; 64];
//    let mut rook_attacks: [u64; 64] = [0; 64];
//    let mut rook_shifts: [u64; 64] = [0; 64];
//    let mut index: [u64; 64];
//
//    let mut bishop_masks: [u64; 64] = [0; 64];
//    let mut bishop_magics: [u64; 64] = [0; 64];
//    let mut bishop_attacks: [u64; 64] = [0; 64];
//    let mut bishop_shifts: [u64; 64] = [0; 64];
//    let bishop_deltas: [i8; 4] = [7,9,9,7];
//    let rook_deltas: [i8; 4] = [8,1,-8,1];
//
//    init_rook_magics(rook_table, rook_attacks, rook_magics, rook_masks, rook_shifts, rook_deltas, index)
//
//}

// std::mem::transmute::<f32, u32>(1.0)
//fn init_rook_magics(mut table: [u64; 0x19000], mut attacks: [&mut [u64]; 64], mut magics: [u64; 64],
//                    mut masks: [u64; 64], mut shifts: [u64; 64], deltas: [i8; 4], mut index: [u64; 64]) {
//
////    let seeds: [[u64;8]; 2] = [ [ 8977, 44560, 54343, 38998,  5731, 95205, 104912, 17020 ],
////                                [  728, 10316, 55013, 32803, 12281, 15100,  16645,   255 ] ];
//
//    let mut occupancy: [u64; 4096] = [0; 4096];
//    let mut reference: [u64; 4096] = [0; 4096];
//    let mut edges: u64 = 0;
//    let mut age: [i32; 4096] =  [0; 4096];
//
//    let mut current: i32 = 0;
//    let mut size: usize = 0;
//
//    attacks[0] = unsafe{std::slice::from_raw_parts_mut(table.as_mut_ptr(),8000)};
////    attacks[0] = table.index_mut(0..8000);
//
//    // s = index for the square
//    for s in 0..64 {
//        // ((Rank1BB | Rank8BB) & ~rank_bb(s)) | ((FileABB | FileHBB) & ~file_bb(s));
//        edges = ((RANK_1 | RANK_8) & !rank_bb(s)) | ((FILE_A | FILE_B) & !file_bb(s));
//
//        masks[s as usize] = sliding_attack(deltas, s, 0) & !edges;
//        shifts[s as usize] = (64 - popcount64(masks[s as usize])) as u64;
//
//        let mut b = 0;
//        size = 0;
//
//        loop {
//            occupancy[size] = b;
//            reference[size] = sliding_attack(deltas, s, b);
//            size += 1;
//            b = (b - masks[s as usize]) * masks[s as usize];
//            if b == 0 {
//                break;
//            }
//        }
//        if s < 63 {
//            unsafe {
//                attacks[s as usize + 1] = std::slice::from_raw_parts_mut(
//                    attacks[s as usize].as_mut_ptr().offset(size as isize),4000);
////                attacks[s as usize] = slice::from_raw_parts(attacks[s as usize], size);
//            }
////            https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html
//        }
//        let mut rng = PRNG::init(seeds[1][rank_of(s) as usize]);
//        'outer: loop {
//            'first_in: loop {
//                magics[s as usize] = rng.sparse_rand();
//                if popcount64((magics[s as usize] * masks[s as usize]) >> 56) < 6 {
//                    break 'first_in;
//                }
//            }
//            // magic_index return unsigned(((occupied & Masks[s]) * Magics[s]) >> Shifts[s]);
//            current += 1;
//            let mut i: usize = 0;
//            'secon_in: while i < size {
//                let index: usize = (((occupancy[i as usize] & masks[s as usize]) * magics[s as usize]) >> shifts[s as usize]) as usize;
//                if age[index] < current {
//                    age[index] = current;
//                    attacks[s as usize][index] = reference[i];
//                } else if attacks[s as usize][index as usize] != reference[i] {
//                    break 'secon_in;
//                }
//            }
//            if i < size {
//                break 'outer;
//            }
//        }
//    }
//}
//// https://bluss.github.io/rust-ndarray/master/ndarray/struct.ArrayBase.html

struct SMagic<'a> {
    ptr: &'a [u64],
    mask: u64,
    magic: u64,
    shift: u32
}

struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32
}

impl PreSMagic {
    pub fn init() -> PreSMagic {
        PreSMagic {start: 0, len: 0, mask: 0, magic: 0, shift: 0}
    }

    pub unsafe fn init64() -> [PreSMagic; 64] {
        let arr: [PreSMagic; 64] = mem::uninitialized();
        arr
    }

    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

struct MRookTable<'a> {
    sq_magics: [SMagic<'a>; 64],
    // attacks: Vec<u64; 0x19000]>
    attacks: Vec<BitBoard>
}

struct MBishopTable<'a> {
    sq_magics: [SMagic<'a>; 64],
    attacks: Box<[u64; 0x1480]>
}

const seeds: [[u64;8]; 2] = [ [ 8977, 44560, 54343, 38998,  5731, 95205, 104912, 17020 ],
[  728, 10316, 55013, 32803, 12281, 15100,  16645,   255 ] ];


// TODO:
impl <'a> MRookTable<'a>  {
    pub fn init() -> MRookTable<'a> {
        let mut pre_sq_table: [PreSMagic; 64] = unsafe {PreSMagic::init64() };
        for i in 0..64 {
            pre_sq_table[i] = PreSMagic::init();
        }
        let mut attacks: Vec<BitBoard> = Vec::with_capacity(102400);

        for i in 0..102400 {
            attacks.push(0);
        }

        let rook_deltas: [i8; 4] = [8,1,-8,1];
        let mut occupancy: [u64; 4096] = [0; 4096];
        let mut reference: [u64; 4096] = [0; 4096];
        let mut age: [i32; 4096] =  [0; 4096];

        let mut size: usize = 0;
        let mut b: u64 = 0;
        let mut current: i32 = 0;
        let mut i: usize = 0;

        pre_sq_table[0].start = 0;

        for s in 0..64 as SQ {
            let mut max = 0;
            println!("{:?}",s);
            let mut magic = 0;
            let edges: BitBoard = ((RANK_1 | RANK_8) & !rank_bb(s)) | ((FILE_A | FILE_H) & !file_bb(s));
            let mask: BitBoard = sliding_attack(rook_deltas, s, 0) & !edges;
            let shift: u32 = (64 - popcount64(mask)) as u32;
            b = 0;
            size = 0;

            'bit: loop {
                occupancy[size] = b;
                reference[size] = sliding_attack(rook_deltas, s, b);
                size += 1;
                b = ((b).wrapping_sub(mask)) as u64 & mask;
                if b == 0 { break 'bit; }
            }
            pre_sq_table[s as usize].len = size;
            if s < 63 {
                pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();

            }
            let mut rng = PRNG::init(seeds[1][rank_of_sq(s) as usize]);

            println!("size: {:?}",size);
            println!("shift {:?}",shift);
            println!("mask {:b}",mask);
            'outer: loop {
                'first_in: loop {
                    magic = rng.sparse_rand();
                    if popcount64((magic.wrapping_mul(mask)).wrapping_shr(56)) >= 6 {
                        break 'first_in;
                    }
                }
//                println!("wemade it {:?}",magic);
                // magic_index return unsigned(((occupied & Masks[s]) * Magics[s]) >> Shifts[s]);
                current += 1;
//                println!("curr {:?}",current);
                i = 0;
                while i < size {
//                    println!("i {:?}",i);
                    let index: usize = ((occupancy[i as usize] & mask).wrapping_mul(magic) as u64).wrapping_shr(shift) as usize;
//                    println!("idx {:?}",index);
                    if age[index] < current {
                        age[index] = current;
                        attacks[pre_sq_table[s as usize].start + index] = reference[i];
                    } else if attacks[pre_sq_table[s as usize].start + index] != reference[i] {
                        break;
                    }
                    i += 1;
                }
//                println!("i {:?} size: {:?}",i,size);
                if i >= size {
                    println!("we got one {:?}", 0);
                    println!("index: {:?}", s);
                    println!("magic: {:?}", magic);
                    break 'outer;
                }
                if i > max {
                    max = i;
                    println!("i {:?} size: {:?}",i,size);
                }
            }
            pre_sq_table[s as usize].magic = magic;
            pre_sq_table[s as usize].mask = mask;
            pre_sq_table[s as usize].shift = shift;
        }
        unsafe {
            let mut sq_table: [SMagic<'a>; 64] = std::mem::uninitialized();
            let mut size = 0;
            for i in 0.. 64 {
                let beginptr = attacks.as_ptr().offset(size as isize);
                let mut table_i: SMagic = SMagic {
                    ptr: mem::uninitialized(),
                    mask: pre_sq_table[i].mask,
                    magic: pre_sq_table[i].magic,
                    shift: pre_sq_table[i].shift,
                };
                table_i.ptr = unsafe {
                    slice::from_raw_parts(beginptr,pre_sq_table[i].len)
//                    attacks.index(Range{ start: pre_sq_table[i].start, end: pre_sq_table[i].next_idx()});
                };
                size += pre_sq_table[i].len;
                sq_table[i] = table_i;

            }
            println!("{:?}",size);
            assert_eq!(size, 102400);
            MRookTable{sq_magics: sq_table, attacks: attacks}
        }

    }
}

struct PRNG {
    seed: u64
}

impl PRNG {
    pub fn init(s: u64) -> PRNG {
        assert_ne!(s,0);
        PRNG {seed: s}
    }

    pub fn rand(&mut self) -> u64 {
        self.rand_change()
    }

    pub fn sparse_rand(&mut self) -> u64 {
        let mut s = self.rand_change();
        s &= self.rand_change();
        s &= self.rand_change();
        s
    }

    fn rand_change(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685821657736338717)
    }
}


//
fn sliding_attack(deltas: [i8; 4], square: SQ, occupied: BitBoard) -> BitBoard {
    assert!(square < 64);
    assert!(square >= 0);
    let sq: i8 = square as i8;
    let mut attack: BitBoard = 0;
    for i in 0..4 as usize {
        let mut s: SQ = ((sq).wrapping_add(deltas[i])) as u8;
        'inner: while sq_is_okay(s) && sq_distance(s, ((s as i8).wrapping_sub(deltas[i])) as u8) == 1 {
            attack = attack | (1 as u64).wrapping_shl(s as u32);
            if occupied & (1 << s) != 0 {break 'inner;}
            s = (s as i8).wrapping_add(deltas[i]) as u8;
        }
    }
    attack
}

pub fn sq_distance(sq1: SQ, sq2: SQ) -> u8 {
    let x = distance(file_of_sq(sq1),file_of_sq(sq2));
    let y = distance(rank_of_sq(sq1),rank_of_sq(sq2));
    cmp::max(x,y)
}

pub fn distance(x: u8, y: u8) -> u8 {
    match x < y {
        true =>  return y - x,
        false => return x - y,
    }
}


pub fn format_bits(bits: String) {
    let x = 64 - bits.len();
    let mut i = 0;
    while i < x {
        print!("0");
        i += 1;
    }
    println!("{}",bits);
}

#[test]
fn test_king_mask_gen() {
    let arr = gen_king_moves().to_vec();
    let sum = arr.iter().fold(0 as  u64,|a, &b| a + (popcount64(b) as u64));
    assert_eq!(sum, (3*4) + (5 * 6 * 4) + (8 * 6 * 6));
}

#[test]
fn test_knight_mask_gen() {
    let arr = gen_knight_moves().to_vec();
    let sum = arr.iter().fold(0 as  u64,|a, &b| a + (popcount64(b) as u64));
    assert_eq!(sum, (2 * 4) + (4 * 4) + (3 * 2 * 4) + (4 * 4 * 4) + (6 * 4 * 4) + (8 * 4 * 4));
}

#[test]
fn occupancy_and_sliding() {
    let rook_deltas: [i8; 4] = [8,1,-8,1];
    assert_eq!(popcount64(sliding_attack(rook_deltas, 0, 0)),14);
    assert_eq!(popcount64(sliding_attack(rook_deltas, 0, 0xFF00)),8);
}

fn edges() {
    let rook_deltas: [i8; 4] = [8,1,-8,1];
    assert_eq!(popcount64(sliding_attack(rook_deltas, 0, 0)),14);
    assert_eq!(popcount64(sliding_attack(rook_deltas, 0, 0xFF00)),8);
}

#[test]
fn rmagics() {
    let mstruct = MRookTable::init();
    assert_eq!(mem::size_of_val(&mstruct), 2584);
}

