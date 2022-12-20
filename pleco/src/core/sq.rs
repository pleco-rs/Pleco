//! Contains the representation of a chessboard's square.
//!
//! Internally, a `SQ` is just a u8. The number of a `SQ` maps to the following
//! squares of a chessboard:
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
//! # Examples
//!
//! ```rust
//! use pleco::core::sq::*;
//! let h1 = SQ::H1;
//! let h2 = SQ::H2;
//!
//! let g2 = SQ(14);
//!
//! assert_eq!(h1.distance(h2), 1);
//! assert_eq!(h1.file(), h2.file());
//! assert_eq!(g2.rank(), h2.rank());
//! ```
//!
//! # Use of `NO_SQ`
//!
//! `NO_SQ` is used to signify the lack of a legal square. Think about this as being a
//! lazy version of `Option<SQ>` where the result is `None`. With normal operation, this
//! shouldn't be a case worth considering.
//!
//! ```rust
//! use pleco::core::sq::*;
//! let no_sq: SQ = NO_SQ;
//! let sq_64 = SQ(64);
//!
//! assert!(!no_sq.is_okay());
//! assert!(!sq_64.is_okay());
//! assert_eq!(no_sq, sq_64);
//! ```
//!
//! # General Safety
//!
//! Generally, all of these methods for a `SQ` are safe to use. The exception to this is
//! when a `SQ::is_okay()` returns false, meaning the square is outside the legal bounds.
//! If methods are used on a square that is not legal, then undefined behavior will follow.

use super::bitboard::BitBoard;
use super::masks::*;
use super::*;

use std::fmt;
use std::mem::transmute;
use std::ops::*;

// TODO: Investigate possibility of using an Enum instead

/// Represents a singular square of a chessboard.
#[derive(Copy, Clone, Default, Hash, PartialEq, PartialOrd, Eq, Debug)]
#[repr(transparent)]
pub struct SQ(pub u8);

impl_bit_ops!(SQ, u8);

/// `SQ` representing no square available. Used internally to represent
/// the lack of an available en-passant square.
pub const NO_SQ: SQ = SQ(64);

impl SQ {
    /// A square that isn't on the board. Basically equivalent to `Option<SQ>` where the value is
    /// `None`.
    pub const NONE: SQ = NO_SQ;

    /// Returns if a `SQ` is within the legal bounds of a square,
    /// which is inclusively between 0 - 63.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::SQ;
    /// let sq_ok = SQ(5);
    /// let no_sq = SQ(64);
    ///
    /// assert!(sq_ok.is_okay());
    /// assert!(!no_sq.is_okay());
    /// ```
    #[inline(always)]
    pub const fn is_okay(self) -> bool {
        self.0 < 64
    }

    /// Returns distance between this square and another square. Distance is
    /// not in algebraic difference, but in squares away.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::SQ;
    ///
    /// let a1 = SQ::A1;
    /// let b2 = SQ::B2;
    /// let b3 = SQ::B3;
    ///
    /// assert_eq!(a1.distance(a1), 0);
    /// assert_eq!(a1.distance(b2), 1);
    /// assert_eq!(a1.distance(b3), 2);
    /// ```
    #[inline]
    pub fn distance(self, sq_other: SQ) -> u8 {
        let x = diff(self.rank_idx_of_sq(), sq_other.rank_idx_of_sq());
        let y = diff(self.file_idx_of_sq(), sq_other.file_idx_of_sq());
        if x > y {
            x
        } else {
            y
        }
    }

    /// Converts a `SQ` to it's `BitBoard` equivalent. The resulting `BitBoard` will
    /// have exactly 1 bit set at the index where the square is location on the
    /// chessboard.
    #[inline(always)]
    pub fn to_bb(self) -> BitBoard {
        assert!(self.is_okay());
        BitBoard(1) << self
    }

    /// Returns the `Rank` that a `SQ` lies on.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::{SQ,Rank};
    ///
    /// let sq_f2 = SQ::F2;
    /// assert_eq!(sq_f2.rank(), Rank::R2);
    /// ```
    #[inline(always)]
    pub fn rank(self) -> Rank {
        //        ALL_RANKS[(self.0 >> 3) as usize]
        unsafe { transmute::<u8, Rank>((self.0 >> 3) & 0b0000_0111) }
    }

    /// Returns the `BitBoard` representation of a `Rank` that a `SQ` lies on.
    #[inline(always)]
    pub fn rank_bb(self) -> BitBoard {
        BitBoard(RANK_BB[self.rank() as usize])
    }

    /// Returns the rank index (number) of a `SQ`.
    #[inline(always)]
    pub const fn rank_idx_of_sq(self) -> u8 {
        (self.0 >> 3) as u8
    }

    /// Returns the `File` that a `SQ` lies on.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::{SQ,File};
    ///
    /// let sq_f2 = SQ::F2;
    /// assert_eq!(sq_f2.file(), File::F);
    /// ```
    #[inline(always)]
    pub fn file(self) -> File {
        unsafe { transmute::<u8, File>(self.0 & 0b0000_0111) }
    }

    /// Returns the `BitBoard` representation of a `File` that a `SQ` lies on.
    #[inline(always)]
    pub fn file_bb(self) -> BitBoard {
        BitBoard(FILE_BB[self.file() as usize])
    }

    /// Returns the file index (number) of a `SQ`.
    #[inline(always)]
    pub const fn file_idx_of_sq(self) -> u8 {
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
            _ => 0,
        }
    }

    /// Creates a `SQ` from the designated File and Rank.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::{SQ,Rank,File};
    ///
    /// let file_f = File::F;
    /// let rank_2 = Rank::R2;
    /// let sq_f2 = SQ::F2;
    ///
    /// assert_eq!(sq_f2, SQ::make(file_f, rank_2));
    /// ```
    #[inline(always)]
    pub fn make(file: File, rank: Rank) -> SQ {
        SQ(((rank as u8).wrapping_shl(3) + (file as u8)) as u8)
    }

    #[inline(always)]
    /// Returns if the `SQ` is a dark square.
    pub fn on_dark_square(self) -> bool {
        (self.to_bb() & BitBoard::DARK_SQUARES).is_empty()
    }

    /// Returns if the `SQ` is a dark square.
    #[inline(always)]
    pub fn on_light_square(self) -> bool {
        (self.to_bb() & BitBoard::DARK_SQUARES).is_not_empty()
    }

    // 0 = white squares, 1 = black square
    /// Returns the player index of the color of the square.
    #[inline(always)]
    pub fn square_color_index(self) -> usize {
        self.on_dark_square() as usize
    }

    /// Flips the square's rank, so `SQ::A1` -> `SQ::A8`.
    #[inline(always)]
    pub fn flip(self) -> SQ {
        SQ(self.0 ^ 0b111000)
    }

    /// Determines if two squares are on opposite colors.
    #[inline(always)]
    pub fn opposite_colors(self, other: SQ) -> bool {
        let s: u8 = self.0 as u8 ^ other.0 as u8;
        ((s >> 3) ^ s) & 1 != 0
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
        write!(f, "{}", SQ_DISPLAY[self.0 as usize])
    }
}
