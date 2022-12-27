use std::mem;
use std::ptr;

use core::masks::*;
use SQ;

use core::bit_twiddles::popcount64;
use core::{file_bb, rank_bb};
use tools::prng::PRNG;

/// Size of the magic rook table.
const ROOK_M_SIZE: usize = 102_400;
static mut ROOK_MAGICS: [SMagic; 64] = [SMagic::init(); 64];
static mut ROOK_TABLE: [u64; ROOK_M_SIZE] = [0; ROOK_M_SIZE];

/// Size of the magic bishop table.
const BISHOP_M_SIZE: usize = 5248;
static mut BISHOP_MAGICS: [SMagic; 64] = [SMagic::init(); 64];
static mut BISHOP_TABLE: [u64; BISHOP_M_SIZE] = [0; BISHOP_M_SIZE];

const B_DELTAS: [i8; 4] = [7, 9, -9, -7];
const R_DELTAS: [i8; 4] = [8, 1, -8, -1];

const SEEDS: [[u64; 8]; 2] = [
    [8977, 44_560, 54_343, 38_998, 5731, 95_205, 104_912, 17_020],
    [728, 10_316, 55_013, 32_803, 12_281, 15_100, 16_645, 255],
];

#[cold]
pub fn init_magics() {
    unsafe {
        gen_magic_board(
            BISHOP_M_SIZE,
            &B_DELTAS,
            BISHOP_MAGICS.as_mut_ptr(),
            BISHOP_TABLE.as_mut_ptr(),
        );
        gen_magic_board(
            ROOK_M_SIZE,
            &R_DELTAS,
            ROOK_MAGICS.as_mut_ptr(),
            ROOK_TABLE.as_mut_ptr(),
        );
    }
}

#[inline]
pub fn bishop_attacks(mut occupied: u64, square: u8) -> u64 {
    let magic_entry: &SMagic = unsafe { BISHOP_MAGICS.get_unchecked(square as usize) };
    occupied &= magic_entry.mask;
    occupied = occupied.wrapping_mul(magic_entry.magic);
    occupied = occupied.wrapping_shr(magic_entry.shift);
    unsafe { *(magic_entry.ptr as *const u64).add(occupied as usize) }
}

#[inline]
pub fn rook_attacks(mut occupied: u64, square: u8) -> u64 {
    let magic_entry: &SMagic = unsafe { ROOK_MAGICS.get_unchecked(square as usize) };
    occupied &= magic_entry.mask;
    occupied = occupied.wrapping_mul(magic_entry.magic);
    occupied = occupied.wrapping_shr(magic_entry.shift);
    unsafe { *(magic_entry.ptr as *const u64).add(occupied as usize) }
}

/// Structure inside a `MagicTable` for a specific hash. For a certain square,
/// contains a mask,  magic number, number to shift by, and a pointer into the array slice
/// where the position is held.
#[derive(Copy, Clone)]
struct SMagic {
    ptr: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl SMagic {
    pub const fn init() -> Self {
        SMagic {
            ptr: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }
}

/// Temporary struct used to create an actual `SMagic` Object.
struct PreSMagic {
    start: usize,
    len: usize,
    mask: u64,
    magic: u64,
    shift: u32,
}

impl PreSMagic {
    pub fn init() -> PreSMagic {
        PreSMagic {
            start: 0,
            len: 0,
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }

    // creates an array of PreSMagic
    pub unsafe fn init64() -> [PreSMagic; 64] {
        let arr: [PreSMagic; 64] = mem::MaybeUninit::uninit().assume_init();
        arr
    }

    // Helper method to compute the next index
    pub fn next_idx(&self) -> usize {
        self.start + self.len
    }
}

/// Creates the `MagicTable` struct. The table size is relative to the piece for computation,
/// and the deltas are the directions on the board the piece can go.
#[cold]
unsafe fn gen_magic_board(
    table_size: usize,
    deltas: &[i8; 4],
    static_magics: *mut SMagic,
    attacks: *mut u64,
) {
    // Creates PreSMagic to hold raw numbers. Technically just adds room to stack
    let mut pre_sq_table: [PreSMagic; 64] = PreSMagic::init64();

    // Initializes each PreSMagic
    for table in pre_sq_table.iter_mut() {
        *table = PreSMagic::init();
    }

    // Occupancy tracks occupancy permutations. MAX permutations = subset of 12 bits = 2^12
    // Reference is similar, tracks the sliding moves from a given occupancy
    // Age tracks the best index for a current permutation
    let mut occupancy: [u64; 4096] = [0; 4096];
    let mut reference: [u64; 4096] = [0; 4096];
    let mut age: [i32; 4096] = [0; 4096];

    // Size tracks the size of permutations of the current block
    let mut size: usize;

    // b is used for generating the permutations through ripple - carry
    let mut b: u64;

    // current and i is a placeholder for actually generating correct magic numbers
    let mut current: i32 = 0;
    let mut i: usize;

    // set the first PreSMagic start = 0. Just in case.
    pre_sq_table[0].start = 0;

    // Loop through each square! s is a SQ
    for s in 0..64_u8 {
        // Magic number for later
        let mut magic: u64;

        // edges is the bitboard representation of the edges s is not on.
        // e.g. sq A1 is on FileA and Rank1, so edges = bitboard of FileH and Rank8
        // mask = occupancy mask of square s
        let edges: u64 = ((RANK_1 | RANK_8) & !rank_bb(s)) | ((FILE_A | FILE_H) & !file_bb(s));
        let mask: u64 = sliding_attack(deltas, s, 0) & !edges;

        // Shift = number of bits in 64 - bits in mask = log2(size)
        let shift: u32 = (64 - popcount64(mask)) as u32;
        b = 0;
        size = 0;

        // Ripple carry to determine occupancy, reference, and size
        'bit: loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(deltas, s, b);
            size += 1;
            b = ((b).wrapping_sub(mask)) as u64 & mask;
            if b == 0 {
                break 'bit;
            }
        }

        // Set current PreSMagic length to be of size
        pre_sq_table[s as usize].len = size;

        // If there is a next square, set the start of it.
        if s < 63 {
            pre_sq_table[s as usize + 1].start = pre_sq_table[s as usize].next_idx();
        }
        // Create our Random Number Generator with a seed
        let mut rng = PRNG::init(SEEDS[1][SQ(s).rank() as usize]);

        // Loop until we have found our magics!
        'outer: loop {
            // Create a magic with our desired number of bits in the first 8 places
            'first_in: loop {
                magic = rng.sparse_rand();
                if popcount64((magic.wrapping_mul(mask)).wrapping_shr(56)) >= 6 {
                    break 'first_in;
                }
            }
            current += 1;
            i = 0;

            // Filling the attacks Vector up to size digits
            while i < size {
                // Magic part! The index is = ((occupancy[s] & mask) * magic >> shift)
                let index: usize = ((occupancy[i as usize] & mask).wrapping_mul(magic) as u64)
                    .wrapping_shr(shift) as usize;

                // Checking to see if we have visited this index already with a lower current number
                if age[index] < current {
                    // If we have visited with lower current, we replace it with this current number,
                    // as this current is higher and has gone through more passes
                    age[index] = current;
                    *attacks.add(pre_sq_table[s as usize].start + index) = reference[i];
                } else if *attacks.add(pre_sq_table[s as usize].start + index) != reference[i] {
                    // If a magic maps to the same index but different result, either magic is bad or we are done
                    break;
                }
                i += 1;
            }
            // If we have filled it up to size or greater, we are done
            if i >= size {
                break 'outer;
            }
        }
        // Set the remaining variables for the PreSMagic Struct
        pre_sq_table[s as usize].magic = magic;
        pre_sq_table[s as usize].mask = mask;
        pre_sq_table[s as usize].shift = shift;
    }

    // size = running total of total size
    let mut size = 0;
    for i in 0..64 {
        // begin ptr points to the beginning of the current slice in the vector
        let beginptr = attacks.add(size);

        // points to the static entry
        let staticptr: *mut SMagic = static_magics.add(i);
        let table_i: SMagic = SMagic {
            ptr: beginptr as usize,
            mask: pre_sq_table[i].mask,
            magic: pre_sq_table[i].magic,
            shift: pre_sq_table[i].shift,
        };

        ptr::copy::<SMagic>(&table_i, staticptr, 1);

        // Create the pointer to the slice with begin_ptr / length
        size += pre_sq_table[i].len;
    }
    // Sanity check
    assert_eq!(size, table_size);
}

/// Returns a bitboards of sliding attacks given an array of 4 deltas/
/// Does not include the original position/
/// Includes occupied bits if it runs into them, but stops before going further.
fn sliding_attack(deltas: &[i8; 4], sq: u8, occupied: u64) -> u64 {
    assert!(sq < 64);
    let mut attack: u64 = 0;
    let square: i16 = sq as i16;
    for delta in deltas.iter().take(4_usize) {
        let mut s: u8 = ((square as i16) + (*delta as i16)) as u8;
        'inner: while s < 64 && SQ(s as u8).distance(SQ(((s as i16) - (*delta as i16)) as u8)) == 1
        {
            attack |= 1_u64.wrapping_shl(s as u32);
            if occupied & 1_u64.wrapping_shl(s as u32) != 0 {
                break 'inner;
            }
            s = ((s as i16) + (*delta as i16)) as u8;
        }
    }
    attack
}
