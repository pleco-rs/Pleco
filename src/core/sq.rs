//! Contains the representation of a chessboard's square.

use super::bitboard::BitBoard;
use super::masks::*;
use super::*;

use std::fmt;
use std::ops::*;

// TODO: Investigate possibility of using an Enum instead after 0.2.0 release.

/// Represents a singular square of a chessboard.
#[derive(Copy, Clone, Default, Hash, PartialEq, PartialOrd, Eq, Debug)]
pub struct SQ(pub u8);

impl_bit_ops!(SQ, u8);

/// `SQ` representing no square available. Used internally to
/// represent the lack of an available en-passant square.
pub const NO_SQ: SQ = SQ(64);

impl SQ {

    /// Returns the UCI String representation of a `SQ`.
    #[inline]
    pub fn to_string(self) -> String {
        assert!(self.is_okay());
        let mut str = String::default();
        str.push(FILE_DISPLAYS[self.file_of_sq() as usize]);
        str.push(RANK_DISPLAYS[self.rank_of_sq() as usize]);
        str
    }

    /// Returns if a `SQ` is within the legal bounds of a square,
    /// which is inclusively between 0 - 63.
    #[inline(always)]
    pub fn is_okay(self) -> bool {
        self.0 < 64
    }

    /// Returns distance between this square and another square. Distance is
    /// not in algebraic difference, but in squares away.
    pub fn distance(self, sq_other: SQ) -> u8 {
        let x = diff(self.rank_idx_of_sq(), sq_other.rank_idx_of_sq());
        let y = diff(self.file_idx_of_sq(), sq_other.file_idx_of_sq());
        if x > y {
           x
        } else {
            y
        }
    }

    /// Converts a `SQ` to it's `BitBoard` equivalent.
    #[inline(always)]
    pub fn to_bb(self) -> BitBoard {
        assert!(self.is_okay());
        BitBoard(1) << self
    }

    /// Returns the `BitBoard` representation of a `Rank` that a `SQ` lies on.
    #[inline(always)]
    pub fn rank_bb(self) -> BitBoard {
        BitBoard(RANK_BB[self.rank_of_sq() as usize])
    }

    /// Returns the `Rank` that a `SQ` lies on.
    #[inline(always)]
    pub fn rank_of_sq(self) -> Rank {
        ALL_RANKS[(self.0 >> 3) as usize]
    }

    /// Returns the rank index (number) of a `SQ`.
    #[inline(always)]
    pub fn rank_idx_of_sq(self) -> u8 {
        (self.0 >> 3) as u8
    }

    /// Returns the `BitBoard` representation of a `File` that a `SQ` lies on.
    #[inline(always)]
    pub fn file_bb(self) -> BitBoard {
        BitBoard(FILE_BB[self.file_of_sq() as usize])
    }

    /// Returns the `File` that a `SQ` lies on.
    #[inline(always)]
    pub fn file_of_sq(self) -> File {
        ALL_FILES[(self.0 & 0b0000_0111) as usize]
    }

    /// Returns the file index (number) of a `SQ`.
    #[inline(always)]
    pub fn file_idx_of_sq(self) -> u8 {
        (self.0 & 0b0000_0111) as u8
    }

    /// Returns the castle rights mask for the given square. If the
    /// square does not have a castle rights mask, returns 0.
    #[inline]
    pub fn castle_rights_mask(self) -> u8 {
        match self.0 {
            ROOK_WHITE_KSIDE_START => C_WHITE_K_MASK,
            ROOK_WHITE_QSIDE_START => C_WHITE_Q_MASK,
            ROOK_BLACK_KSIDE_START => C_BLACK_K_MASK,
            ROOK_BLACK_QSIDE_START => C_BLACK_Q_MASK,
            WHITE_KING_START => C_WHITE_K_MASK | C_WHITE_Q_MASK,
            BLACK_KING_START => C_BLACK_K_MASK | C_BLACK_Q_MASK,
            _ => 0
        }
    }

    /// Creates a `SQ` from the designated File and Rank.
    #[inline]
    pub fn make(file: File, rank: Rank) -> SQ {
        SQ(((rank as u8).wrapping_shl(3) + (file as u8)) as u8)
    }
}

// constants
impl SQ {
    #[doc(hidden)]
    pub const NO_SQ: SQ = NO_SQ;
    #[doc(hidden)]
    pub const A1: SQ = SQ(0b000000);
    #[doc(hidden)]
    pub const B1: SQ = SQ(0b000001);
    #[doc(hidden)]
    pub const C1: SQ = SQ(0b000010);
    #[doc(hidden)]
    pub const D1: SQ = SQ(0b000011);
    #[doc(hidden)]
    pub const E1: SQ = SQ(0b000100);
    #[doc(hidden)]
    pub const F1: SQ = SQ(0b000101);
    #[doc(hidden)]
    pub const G1: SQ = SQ(0b000110);
    #[doc(hidden)]
    pub const H1: SQ = SQ(0b000111);
    #[doc(hidden)]
    pub const A2: SQ = SQ(0b001000);
    #[doc(hidden)]
    pub const B2: SQ = SQ(0b001001);
    #[doc(hidden)]
    pub const C2: SQ = SQ(0b001010);
    #[doc(hidden)]
    pub const D2: SQ = SQ(0b001011);
    #[doc(hidden)]
    pub const E2: SQ = SQ(0b001100);
    #[doc(hidden)]
    pub const F2: SQ = SQ(0b001101);
    #[doc(hidden)]
    pub const G2: SQ = SQ(0b001110);
    #[doc(hidden)]
    pub const H2: SQ = SQ(0b001111);
    #[doc(hidden)]
    pub const A3: SQ = SQ(0b010000);
    #[doc(hidden)]
    pub const B3: SQ = SQ(0b010001);
    #[doc(hidden)]
    pub const C3: SQ = SQ(0b010010);
    #[doc(hidden)]
    pub const D3: SQ = SQ(0b010011);
    #[doc(hidden)]
    pub const E3: SQ = SQ(0b010100);
    #[doc(hidden)]
    pub const F3: SQ = SQ(0b010101);
    #[doc(hidden)]
    pub const G3: SQ = SQ(0b010110);
    #[doc(hidden)]
    pub const H3: SQ = SQ(0b010111);
    #[doc(hidden)]
    pub const A4: SQ = SQ(0b011000);
    #[doc(hidden)]
    pub const B4: SQ = SQ(0b011001);
    #[doc(hidden)]
    pub const C4: SQ = SQ(0b011010);
    #[doc(hidden)]
    pub const D4: SQ = SQ(0b011011);
    #[doc(hidden)]
    pub const E4: SQ = SQ(0b011100);
    #[doc(hidden)]
    pub const F4: SQ = SQ(0b011101);
    #[doc(hidden)]
    pub const G4: SQ = SQ(0b011110);
    #[doc(hidden)]
    pub const H4: SQ = SQ(0b011111);
    #[doc(hidden)]
    pub const A5: SQ = SQ(0b100000);
    #[doc(hidden)]
    pub const B5: SQ = SQ(0b100001);
    #[doc(hidden)]
    pub const C5: SQ = SQ(0b100010);
    #[doc(hidden)]
    pub const D5: SQ = SQ(0b100011);
    #[doc(hidden)]
    pub const E5: SQ = SQ(0b100100);
    #[doc(hidden)]
    pub const F5: SQ = SQ(0b100101);
    #[doc(hidden)]
    pub const G5: SQ = SQ(0b100110);
    #[doc(hidden)]
    pub const H5: SQ = SQ(0b100111);
    #[doc(hidden)]
    pub const A6: SQ = SQ(0b101000);
    #[doc(hidden)]
    pub const B6: SQ = SQ(0b101001);
    #[doc(hidden)]
    pub const C6: SQ = SQ(0b101010);
    #[doc(hidden)]
    pub const D6: SQ = SQ(0b101011);
    #[doc(hidden)]
    pub const E6: SQ = SQ(0b101100);
    #[doc(hidden)]
    pub const F6: SQ = SQ(0b101101);
    #[doc(hidden)]
    pub const G6: SQ = SQ(0b101110);
    #[doc(hidden)]
    pub const H6: SQ = SQ(0b101111);
    #[doc(hidden)]
    pub const A7: SQ = SQ(0b110000);
    #[doc(hidden)]
    pub const B7: SQ = SQ(0b110001);
    #[doc(hidden)]
    pub const C7: SQ = SQ(0b110010);
    #[doc(hidden)]
    pub const D7: SQ = SQ(0b110011);
    #[doc(hidden)]
    pub const E7: SQ = SQ(0b110100);
    #[doc(hidden)]
    pub const F7: SQ = SQ(0b110101);
    #[doc(hidden)]
    pub const G7: SQ = SQ(0b110110);
    #[doc(hidden)]
    pub const H7: SQ = SQ(0b110111);
    #[doc(hidden)]
    pub const A8: SQ = SQ(0b111000);
    #[doc(hidden)]
    pub const B8: SQ = SQ(0b111001);
    #[doc(hidden)]
    pub const C8: SQ = SQ(0b111010);
    #[doc(hidden)]
    pub const D8: SQ = SQ(0b111011);
    #[doc(hidden)]
    pub const E8: SQ = SQ(0b111100);
    #[doc(hidden)]
    pub const F8: SQ = SQ(0b111101);
    #[doc(hidden)]
    pub const G8: SQ = SQ(0b111110);
    #[doc(hidden)]
    pub const H8: SQ = SQ(0b111111);
}

impl fmt::Display for SQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&self.to_string())
    }
}

