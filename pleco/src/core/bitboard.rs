//! Module containing the `BitBoard` and associated functions / constants.
//!
//! A [`BitBoard`] is a set of bits, where the index of each bit represents a square on the
//! Board. We use this to mark whether or not something is residing at a certain square. For
//! example, if we are using it to map the positions of the white pawns, and there exists a
//! pawn at square B2, the bit at index 9 will be set to '1'. The lack of a piece is marked
//! with a '0' instead.
//!
//! Each bit's index of a [`BitBoard`] maps to the following squares:
//!
//! ```md,ignore
//! 8 | 56 57 58 59 60 61 62 63
//! 7 | 48 49 50 51 52 53 54 55
//! 6 | 40 41 42 43 44 45 46 47
//! 5 | 32 33 34 35 36 37 38 39
//! 4 | 24 25 26 27 28 29 30 31
//! 3 | 16 17 18 19 20 21 22 23
//! 2 | 8  9  10 11 12 13 14 15
//! 1 | 0  1  2  3  4  5  6  7
//!   -------------------------
//!      a  b  c  d  e  f  g  h
//! ```
//!
//! [`BitBoard`]: struct.BitBoard.html

extern crate rand;

use super::bit_twiddles::*;
use super::masks::*;
use super::sq::SQ;
use super::Player;
use tools::prng::PRNG;

use std::fmt;
use std::hint::unreachable_unchecked;
use std::mem;
use std::ops::*;

/// A `BitBoard` is simply a 64 bit long integer where each
/// bit maps to a specific square. Used for mapping occupancy, where '1' represents
/// a piece being at that index's square, and a '0' represents a lack of a piece.
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct BitBoard(pub u64);

impl_bit_ops!(BitBoard, u64);

impl BitBoard {
    /// BitBoard of File A.
    pub const FILE_A: BitBoard = BitBoard(FILE_A);
    /// BitBoard of File B.
    pub const FILE_B: BitBoard = BitBoard(FILE_B);
    /// BitBoard of File C.
    pub const FILE_C: BitBoard = BitBoard(FILE_C);
    /// BitBoard of File D.
    pub const FILE_D: BitBoard = BitBoard(FILE_D);
    /// BitBoard of File E.
    pub const FILE_E: BitBoard = BitBoard(FILE_E);
    /// BitBoard of File F.
    pub const FILE_F: BitBoard = BitBoard(FILE_F);
    /// BitBoard of File G.
    pub const FILE_G: BitBoard = BitBoard(FILE_G);
    /// BitBoard of File H.
    pub const FILE_H: BitBoard = BitBoard(FILE_H);
    /// BitBoard of Rank 1.
    pub const RANK_1: BitBoard = BitBoard(RANK_1);
    /// BitBoard of Rank 2.
    pub const RANK_2: BitBoard = BitBoard(RANK_2);
    /// BitBoard of Rank 3.
    pub const RANK_3: BitBoard = BitBoard(RANK_3);
    /// BitBoard of Rank 4.
    pub const RANK_4: BitBoard = BitBoard(RANK_4);
    /// BitBoard of Rank 5.
    pub const RANK_5: BitBoard = BitBoard(RANK_5);
    /// BitBoard of Rank 6.
    pub const RANK_6: BitBoard = BitBoard(RANK_6);
    /// BitBoard of Rank 7.
    pub const RANK_7: BitBoard = BitBoard(RANK_7);
    /// BitBoard of Rank 8.
    pub const RANK_8: BitBoard = BitBoard(RANK_8);

    /// BitBoard of all dark squares.
    pub const DARK_SQUARES: BitBoard = BitBoard(DARK_SQUARES);
    /// BitBoard of all light squares.
    pub const LIGHT_SQUARES: BitBoard = BitBoard(LIGHT_SQUARES);
    /// BitBoard of all squares.
    pub const ALL: BitBoard = BitBoard(!0);

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

    /// Returns the most significant bit
    #[inline(always)]
    pub fn msb(self) -> BitBoard {
        BitBoard(msb(self.0))
    }

    /// Returns the least significant bit as a u64.
    #[inline(always)]
    pub fn lsb_u64(self) -> u64 {
        lsb(self.0)
    }

    /// Returns the index (as a square) of the least significant bit and removes
    /// that bit from the `BitBoard`.
    ///
    /// # Safety
    ///
    /// Panics if the `BitBoard` is empty. See [`BitBoard::pop_some_lsb`] for a
    /// non-panicking version of the method.
    ///
    /// [`BitBoard::pop_some_lsb`]: struct.BitBoard.html#method.pop_some_lsb
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> SQ {
        let sq = self.bit_scan_forward();
        *self &= *self - 1;
        sq
    }

    /// Returns the least significant bit of a `BitBoard`, if it has any. If there is a bit to
    /// return, it removes that bit from itself.
    #[inline(always)]
    pub fn pop_some_lsb(&mut self) -> Option<SQ> {
        if self.is_empty() {
            None
        } else {
            Some(self.pop_lsb())
        }
    }

    /// Returns the index (as a square) and bit of the least significant bit and removes
    /// that bit from the `BitBoard`.
    ///
    /// # Safety
    ///
    /// Panics if the `BitBoard` is empty. See [`BitBoard::pop_some_lsb_and_bit`] for a
    /// non-panicking version of the method.
    ///
    /// [`BitBoard::pop_some_lsb_and_bit`]: struct.BitBoard.html#method.pop_some_lsb_and_bit
    #[inline(always)]
    pub fn pop_lsb_and_bit(&mut self) -> (SQ, BitBoard) {
        let sq: SQ = self.bit_scan_forward();
        *self &= *self - 1;
        (sq, sq.to_bb())
    }

    /// Returns the index (as a square) and bit of the least significant bit and removes
    /// that bit from the `BitBoard`. If there are no bits left (the board is empty), returns
    /// `None`.
    #[inline(always)]
    pub fn pop_some_lsb_and_bit(&mut self) -> Option<(SQ, BitBoard)> {
        if self.is_empty() {
            None
        } else {
            Some(self.pop_lsb_and_bit())
        }
    }

    /// Returns the front-most square of a player on the current `BitBoard`.
    ///
    /// # Safety
    ///
    /// Panics if the `BitBoard` is empty.
    #[inline]
    pub fn frontmost_sq(self, player: Player) -> SQ {
        match player {
            Player::White => self.msb().to_sq(),
            Player::Black => self.bit_scan_forward(),
        }
    }

    /// Returns the back-most square of a player on the current `BitBoard`.
    ///
    /// # Safety
    ///
    /// panics if the `BitBoard` is empty.
    #[inline]
    pub fn backmost_sq(self, player: Player) -> SQ {
        match player {
            Player::White => self.bit_scan_forward(),
            Player::Black => self.msb().to_sq(),
        }
    }

    /// Returns a clone of a `[[BitBoard; 6]; 2]`. Used to duplicate occupancy `BitBoard`s of each
    /// piece for each player.
    #[inline(always)]
    pub fn clone_all_occ(
        bbs: &[[BitBoard; PIECE_TYPE_CNT]; PLAYER_CNT],
    ) -> [[BitBoard; PIECE_TYPE_CNT]; PLAYER_CNT] {
        let new_bbs: [[BitBoard; PIECE_TYPE_CNT]; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }

    /// Returns a clone of a `[BitBoard; 2]`. Used to duplicate occupancy `BitBoard`s of each player.
    #[inline(always)]
    pub fn clone_occ_bbs(bbs: &[BitBoard; PLAYER_CNT]) -> [BitBoard; PLAYER_CNT] {
        let new_bbs: [BitBoard; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
        new_bbs
    }
}

impl Shl<SQ> for BitBoard {
    type Output = BitBoard;

    #[inline(always)]
    fn shl(self, rhs: SQ) -> BitBoard {
        BitBoard((self.0).wrapping_shl(rhs.0 as u32))
    }
}

impl Iterator for BitBoard {
    type Item = SQ;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.pop_lsb())
        }
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = &string_u64(reverse_bytes(self.0));
        f.pad(s)
    }
}

/// Sets the Number of random bits on a randomly-generated `BitBoard`.
#[derive(Eq, PartialEq)]
enum RandAmount {
    VeryDense,       // Average 48 bits
    Dense,           // Average 32 bits
    Standard,        // Average 16 bits
    Sparse,          // Average 8 bits
    VerySparse,      // Average 6 bits
    ExtremelySparse, // Average 4 bits
    Singular,        // One and only one bit set.
}

/// BitBoard generating structure.
pub struct RandBitBoard {
    prng: PRNG,
    seed: u64,
    rand: RandAmount,
    max: u16,
    min: u16,
}

impl Default for RandBitBoard {
    fn default() -> Self {
        RandBitBoard {
            prng: PRNG::init(1),
            seed: 0,
            rand: RandAmount::Standard,
            max: 64,
            min: 1,
        }
    }
}

impl RandBitBoard {
    /// Returns a vector of "amount" BitBoards.
    pub fn many(mut self, amount: usize) -> Vec<BitBoard> {
        let mut boards: Vec<BitBoard> = Vec::with_capacity(amount);
        for _x in 0..amount {
            boards.push(self.go());
        }
        boards
    }

    /// Returns a singular random BitBoard.
    pub fn one(mut self) -> BitBoard {
        self.go()
    }

    /// Sets the average number of bits in the resulting Bitboard.
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

    /// Allows empty BitBoards to be returned.
    pub fn allow_empty(mut self) -> Self {
        self.min = 0;
        self
    }

    /// Sets the maximum number of bits in a `BitBoard`.
    pub fn max(mut self, max: u16) -> Self {
        self.max = max;
        self
    }

    /// Sets the minimum number of bits in a `BitBoard`.
    pub fn min(mut self, min: u16) -> Self {
        self.min = min;
        self
    }

    /// Sets the generation to use pseudo-random numbers instead of random
    /// numbers. The seed is a random number for the random numbers to be generated
    /// off of.
    pub fn pseudo_random(mut self, seed: u64) -> Self {
        self.seed = if seed == 0 { 1 } else { seed };
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
                RandAmount::Dense => self.prng.rand(),                        // Average 32 bits
                RandAmount::Standard => self.prng.rand() & self.prng.rand(),  // Average 16 bits
                RandAmount::Sparse => self.prng.sparse_rand(),                // Average 8 bits
                RandAmount::VerySparse => {
                    self.prng.sparse_rand() & (self.prng.rand() | self.prng.rand())
                } // Average 6 bits
                RandAmount::ExtremelySparse => self.prng.sparse_rand() & self.prng.rand(), // Average 4 bits
                RandAmount::Singular => unsafe { unreachable_unchecked() },
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bb_pop_lsb() {
        let mut bbs = RandBitBoard::default()
            .pseudo_random(2264221)
            .min(2)
            .avg(5)
            .max(15)
            .many(100);

        while !bbs.is_empty() {
            let mut bb = bbs.pop().unwrap();
            while bb.is_not_empty() {
                let total_pre = bb.count_bits();
                let lsb_sq = bb.pop_lsb();
                assert!(lsb_sq.is_okay());
                assert_eq!(lsb_sq.to_bb() & bb, BitBoard(0));
                assert_eq!(bb.count_bits() + 1, total_pre);
            }
        }
    }

    #[test]
    fn rand_bb_gen_eq() {
        let mut bbs_1 = RandBitBoard::default()
            .pseudo_random(9010555142588)
            .avg(16)
            .many(1000);

        let mut bbs_2 = RandBitBoard::default()
            .pseudo_random(9010555142588)
            .avg(16)
            .many(1000);

        assert_eq!(bbs_1.len(), bbs_2.len());
        while !bbs_1.is_empty() {
            assert_eq!(bbs_1.pop(), bbs_2.pop());
        }
    }
}
