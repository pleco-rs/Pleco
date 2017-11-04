//! Module containing the `BitBoard` and associated functions / constants.

extern crate rand;

use super::sq::SQ;
use super::bit_twiddles::*;
use super::masks::*;
use tools::prng::PRNG;

use std::mem;
use std::ops::*;
use std::fmt;

/// Defines an object to define a bitboard. a `BitBoard` is simply a u64 where each
/// bit maps to a specific square. Used for mapping occupancy, where '1' represents
/// a piece being at that index's square, and a '0' represents a lack of a piece.
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
pub struct BitBoard(pub u64);

impl_bit_ops!(BitBoard, u64);

impl BitBoard {

    pub const FILE_A: BitBoard = BitBoard(FILE_A);
    pub const FILE_B: BitBoard = BitBoard(FILE_B);
    pub const FILE_C: BitBoard = BitBoard(FILE_C);
    pub const FILE_D: BitBoard = BitBoard(FILE_D);
    pub const FILE_E: BitBoard = BitBoard(FILE_E);
    pub const FILE_F: BitBoard = BitBoard(FILE_F);
    pub const FILE_G: BitBoard = BitBoard(FILE_G);
    pub const FILE_H: BitBoard = BitBoard(FILE_H);
    pub const RANK_1: BitBoard = BitBoard(RANK_1);
    pub const RANK_2: BitBoard = BitBoard(RANK_2);
    pub const RANK_3: BitBoard = BitBoard(RANK_3);
    pub const RANK_4: BitBoard = BitBoard(RANK_4);
    pub const RANK_5: BitBoard = BitBoard(RANK_5);
    pub const RANK_6: BitBoard = BitBoard(RANK_6);
    pub const RANK_7: BitBoard = BitBoard(RANK_7);
    pub const RANK_8: BitBoard = BitBoard(RANK_8);

    /// Converts a `BitBoard` to a square.
    ///
    /// # Safety
    ///
    /// The `BitBoard` must have exactly one bit inside of it, or else
    /// this will return the square of the least significant bit.
    #[inline(always)]
    pub fn to_sq(self) -> SQ {
        debug_assert_eq!(self.count_bits(), 1);
        SQ(bit_scan_forward(self.0))
    }

    /// Returns the number of bits in a `BitBoard`
    #[inline(always)]
    pub fn count_bits(self) -> u8 {
        popcount64(self.0)
    }

    /// Returns the `SQ` of the least significant bit.
    ///
    /// # Panic
    ///
    /// Will panic if the `BitBoard` is empty.
    #[inline(always)]
    pub fn bit_scan_forward(self) -> SQ {
        SQ(self.bit_scan_forward_u8())
    }

    /// Returns the index (u8) of the least significant bit.
    ///
    /// # Panic
    ///
    /// Will panic if the `BitBoard` is empty.
    #[inline(always)]
    pub fn bit_scan_forward_u8(self) -> u8 {
        assert!(self.is_not_empty());
        bit_scan_forward(self.0)
    }

    /// Returns if there are more than 1 bits inside.
    #[inline(always)]
    pub fn more_than_one(self) -> bool {
        more_than_one(self.0)
    }

    /// Determines if the `BitBoard` is empty (contains no bits).
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Determines if the `BitBoard` is not empty (contains one or more bits).
    #[inline(always)]
    pub fn is_not_empty(self) -> bool {
        self.0 != 0
    }

    /// Returns the least significant bit as a BitBoard.
    #[inline(always)]
    pub fn lsb(self) -> BitBoard {
        BitBoard(self.lsb_u64())
    }

    /// Returns the least significant bit as a u64.
    #[inline(always)]
    pub fn lsb_u64(self) -> u64 {
        lsb(self.0)
    }

    /// Array containing all the `BitBoards` for of the starting position, for each player and piece.
    pub fn start_bbs() -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
        [[
            BitBoard(START_W_PAWN),
            BitBoard(START_W_KNIGHT),
            BitBoard(START_W_BISHOP),
            BitBoard(START_W_ROOK),
            BitBoard(START_W_QUEEN),
            BitBoard(START_W_KING),
        ], [
            BitBoard(START_B_PAWN),
            BitBoard(START_B_KNIGHT),
            BitBoard(START_B_BISHOP),
            BitBoard(START_B_ROOK),
            BitBoard(START_B_QUEEN),
            BitBoard(START_B_KING),
        ], ]
    }

    /// Returns a clone of a `[[BitBoard; 6]; 2]`. Used to duplicate occupancy `BitBoard`s of each
    /// piece for each player.
    #[inline(always)]
    pub fn clone_all_occ(bbs: &[[BitBoard; PIECE_CNT]; PLAYER_CNT], ) -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
        let new_bbs: [[BitBoard; PIECE_CNT]; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }

    /// Returns a clone of a `[BitBoard; 2]`. Used to duplicate occupancy `BitBoard`s of each player.
    #[inline(always)]
    pub fn clone_occ_bbs(bbs: &[BitBoard; PLAYER_CNT]) -> [BitBoard; PLAYER_CNT] {
        let new_bbs: [BitBoard; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }
}

//#[inline(always)]
//pub fn to_bb(self) -> BitBoard {
//    assert!(self.is_okay());
//    BitBoard((1 as u64).wrapping_shl(self.0 as u32))
//}

impl Shl<SQ> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: SQ) -> BitBoard {
        BitBoard((self.0).wrapping_shl(rhs.0 as u32))
    }
}


impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = &string_u64(reverse_bytes(self.0));
        f.pad(s)
    }
}

#[derive(Eq, PartialEq)]
enum RandAmount {
    VeryDense, // Average 48 bits
    Dense,    // Average 32 bits
    Standard,  // Average 16 bits
    Sparse,   // Average 8 bits
    VerySparse, // Average 6 bits
    ExtremelySparse, // Average 4 bits
    Singular // One and only one bit set.
}

pub struct RandBitBoard {
    prng: PRNG,
    seed: u64,
    rand: RandAmount,
    max: u16,
    min: u16
}

impl Default for RandBitBoard {
    fn default() -> Self {
        RandBitBoard {
            prng: PRNG::init(1),
            seed: 0,
            rand: RandAmount::Standard,
            max: 64,
            min: 1
        }
    }
}

impl RandBitBoard {
    pub fn many(mut self, amount: usize) -> Vec<BitBoard> {
        let mut boards: Vec<BitBoard> = Vec::with_capacity(amount);
        for _x in 0..amount {
            boards.push(self.go());
        };
        boards
    }

    pub fn one(mut self) -> BitBoard {
        self.go()
    }

    pub fn avg(mut self, bits: u8) -> Self {
        self.rand = if bits >= 36 {
            RandAmount::VeryDense
        } else if bits >= 26 {
            RandAmount::Dense
        } else if bits >= 12 {
            RandAmount::Standard
        } else if bits >= 7 {
            RandAmount::Sparse
        } else if bits >= 5 {
            RandAmount::VerySparse
        } else {
            RandAmount::ExtremelySparse
        };
        self
    }

    pub fn allow_empty(mut self) -> Self {
        self.min = 0;
        self
    }

    pub fn max(mut self, max: u16) -> Self {
        self.max = max;
        self
    }

    pub fn min(mut self, min: u16) -> Self {
        self.min = min;
        self
    }

    pub fn pseudo_random(mut self, seed: u64) -> Self {
        self.seed = if seed == 0 {1} else {seed};
        self.prng = PRNG::init(seed);
        self
    }

    fn go(&mut self) -> BitBoard {
        if self.rand == RandAmount::Singular {
            return BitBoard(self.prng.singular_bit());
        }

        loop {
            let num = match self.rand {
                RandAmount::VeryDense => self.prng.rand() | self.prng.rand(), // Average 48 bits
                RandAmount::Dense => self.prng.rand(),    // Average 32 bits
                RandAmount::Standard => self.prng.rand() & self.prng.rand(),  // Average 16 bits
                RandAmount::Sparse => self.prng.sparse_rand(),   // Average 8 bits
                RandAmount::VerySparse => self.prng.sparse_rand() & (self.prng.rand() | self.prng.rand()), // Average 6 bits
                RandAmount::ExtremelySparse => self.prng.sparse_rand() & self.prng.rand(),   // Average 4 bits
                RandAmount::Singular => unreachable!()
            };
            let count = popcount64(num) as u16;
            if count >= self.min && count <= self.max {
                return BitBoard(num);
            }
        }
    }

    fn random(&mut self) -> usize {
        if self.seed == 0 {
            return rand::random::<usize>();
        }
        self.prng.rand() as usize
    }
}