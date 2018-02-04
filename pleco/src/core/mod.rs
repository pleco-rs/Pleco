//! Contains various components and structures supporting the creation of a chessboard. This
//! includes `SQ`, `BitBoard`, `Player`, `Piece`, `GenTypes`, `Rank`, and `File`. Also holds
//! the statically created `MagicHelper`, which at runtime creates various lookup tables.

#[macro_use]
mod macros;

pub mod bit_twiddles;
pub mod piece_move;
pub mod magic_helper;
pub mod masks;
pub mod mono_traits;
pub mod sq;
pub mod bitboard;
pub mod move_list;
pub mod score;

use self::bit_twiddles::*;
use self::masks::*;
use self::sq::SQ;

use std::fmt;
use std::mem;
use std::ops::Not;

/// Array of all possible pieces, indexed by their enum value.
pub const ALL_PIECE_TYPES: [PieceType; PIECE_TYPE_CNT] =
    [PieceType::P, PieceType::N, PieceType::B, PieceType::R, PieceType::Q, PieceType::K];


/// Array of both players, indexed by their enum value.
pub const ALL_PLAYERS: [Player; 2] = [Player::White, Player::Black];

/// Array of all `Files`s, indexed by their enum value.
pub static ALL_FILES: [File; FILE_CNT] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

/// Array of all `Rank`s, indexed by their enum value.
pub static ALL_RANKS: [Rank; RANK_CNT] = [
    Rank::R1,
    Rank::R2,
    Rank::R3,
    Rank::R4,
    Rank::R5,
    Rank::R6,
    Rank::R7,
    Rank::R8,
];


/// Enum to represent the Players White & Black.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    /// Returns the other player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::Player;
    ///
    /// let b = Player::Black;
    /// assert_eq!(b.other_player(), Player::White);
    /// ```
    #[inline(always)]
    pub fn other_player(&self) -> Player {
        match *self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    /// Returns the relative square from a given square.
    #[inline(always)]
    pub fn relative_square(&self, sq: SQ) -> SQ {
        assert!(sq.is_okay());
        sq ^ SQ((*self) as u8 * 56)
    }

    /// Gets the direction of a pawn push for a given player.
    #[inline(always)]
    pub fn pawn_push(&self) -> i8 {
        match *self {
            Player::White => NORTH,
            Player::Black => SOUTH,
        }
    }

    /// Returns the relative rank of a square in relation to a player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::{Player,Rank};
    /// use pleco::core::sq::SQ;
    ///
    /// let w = Player::White;
    /// let b = Player::Black;
    ///
    /// assert_eq!(w.relative_rank_of_sq(SQ::A1), Rank::R1);
    /// assert_eq!(b.relative_rank_of_sq(SQ::H8), Rank::R1);
    /// assert_eq!(b.relative_rank_of_sq(SQ::A1), Rank::R8);
    /// ```
    #[inline(always)]
    pub fn relative_rank_of_sq(&self, sq: SQ) -> Rank {
        self.relative_rank(sq.rank())
    }

    /// Returns the relative rank of a rank in relation to a player.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::{Player,Rank};
    /// use pleco::core::sq::SQ;
    ///
    /// let w = Player::White;
    /// let b = Player::Black;
    ///
    /// assert_eq!(w.relative_rank(Rank::R1), Rank::R1);
    /// assert_eq!(b.relative_rank(Rank::R8), Rank::R1);
    /// assert_eq!(b.relative_rank(Rank::R1), Rank::R8);
    /// ```
    #[inline(always)]
    pub fn relative_rank(&self, rank: Rank) -> Rank {
        ALL_RANKS[((rank as u8) ^ (*self as u8 * 7)) as usize]
    }
}


impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if self == &Player::White {
                "White"
            } else {
                "Black"
            }
        )
    }
}

/// Types of move generating options.
///
/// `GenTypes::All` -> All available moves.
///
/// `GenTypes::Captures` -> All captures and both capture/non-capture promotions.
///
/// `GenTypes::Quiets` -> All non captures and both capture/non-capture promotions.
///
/// `GenTypes::QuietChecks` -> Moves likely to give check.
///
/// `GenTypes::Evasions` -> Generates evasions for a board in check.
///
/// `GenTypes::NonEvasions` -> Generates all moves for a board not in check.
///
/// # Safety
///
/// `GenTypes::QuietChecks` and `GenTypes::NonEvasions` can only be used if the board
/// if not in check, while `GenTypes::Evasions` can only be used if the the board is
/// in check. The remaining `GenTypes` can be used legally whenever.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GenTypes {
    All,
    Captures,
    Quiets,
    QuietChecks,
    Evasions,
    NonEvasions
}

/// All possible Types of Pieces on a chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    K = 5,
    Q = 4,
    R = 3,
    B = 2,
    N = 1,
    P = 0,
}

impl PieceType {
    /// Returns the relative value of a piece.
    ///
    /// Used for sorting moves.
    #[inline]
    pub fn value(&self) -> i8 {
        match *self {
            PieceType::P => 1,
            PieceType::N | PieceType::B => 3,
            PieceType::R => 5,
            PieceType::Q => 8,
            PieceType::K => 0,
        }
    }

    /// Return the lowercase character of a `Piece`.
    #[inline]
    pub fn char_lower(&self) -> char {
        match *self {
            PieceType::P => 'p',
            PieceType::N => 'n',
            PieceType::B => 'b',
            PieceType::R => 'r',
            PieceType::Q => 'q',
            PieceType::K => 'k',
        }
    }

    /// Return the uppercase character of a `Piece`.
    #[inline]
    pub fn char_upper(&self) -> char {
        match *self {
            PieceType::P => 'P',
            PieceType::N => 'N',
            PieceType::B => 'B',
            PieceType::R => 'R',
            PieceType::Q => 'Q',
            PieceType::K => 'K',
        }
    }

}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            PieceType::P => "Pawn",
            PieceType::N => "Knight",
            PieceType::B => "Bishop",
            PieceType::R => "Rook",
            PieceType::Q => "Queen",
            PieceType::K => "King"
        };
        f.pad(s)
    }
}

/// Enum for the Files of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, Ord, PartialOrd, Eq)]
pub enum File {
    A = 0, // eg a specific coloumn
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {

    /// Returns the bit-set of all files to the left of the current file.
    #[inline(always)]
    pub fn left_side_mask(self) -> u8 {
        (1 << self as u8) - 1
    }

    /// Returns the bit-set of all files to the right of the current file.
    #[inline(always)]
    pub fn right_side_mask(self) -> u8 {
        !((1 << (self as u16 + 1)) - 1) as u8
    }

    /// Returns the minimum file.
    ///
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::File;
    ///
    /// let file_a = File::A;
    ///
    /// assert_eq!(file_a.min(File::C), File::A);
    /// ```
    pub fn min(self, other: File) -> File {
        if (self as u8) < (other as u8) {
            self
        } else {
            other
        }
    }

    /// Returns the maximum file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::core::File;
    ///
    /// let file_a = File::A;
    ///
    /// assert_eq!(file_a.max(File::C), File::C);
    /// ```
    pub fn max(self, other: File) -> File {
        if (self as u8) > (other as u8) {
            self
        } else {
            other
        }
    }
}

impl Not for File {
    type Output = File;

    fn not(self) -> File {
        unsafe {
            let f = self as u8 ^ File::H as u8;
            mem::transmute::<u8,File>(0b111 & f)
        }
    }
}

/// Enum for the Ranks of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, Eq, Ord, PartialOrd)]
pub enum Rank { // eg a specific row
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

/// Types of Castling available to a player.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum CastleType {
    KingSide = 0,
    QueenSide = 1,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Phase {
    MG = 0,
    EG = 1
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding rank as a u64.
#[inline(always)]
pub fn rank_bb(s: u8) -> u64 {
    RANK_BB[rank_of_sq(s) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank`.
#[inline(always)]
pub fn rank_of_sq(s: u8) -> Rank {
    unsafe {
        mem::transmute::<u8,Rank>((s >> 3) & 0b0000_0111)
    }
//    ALL_RANKS[(s >> 3) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank` index.
#[inline(always)]
pub fn rank_idx_of_sq(s: u8) -> u8 {
    (s >> 3) as u8
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding file as a u64.
#[inline(always)]
pub fn file_bb(s: u8) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File`.
#[inline(always)]
pub fn file_of_sq(s: u8) -> File {
    unsafe {
        mem::transmute::<u8,File>(s & 0b0000_0111)
    }
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File` index.
#[inline(always)]
pub fn file_idx_of_sq(s: u8) -> u8 {
    (s & 0b0000_0111) as u8
}

/// Converts a singular bit of a u64 to it's index in the u64.
/// If there's more than one bit in the u64, this will be done for
/// the least significant bit.
///
/// # Safety
///
/// Undefined behavior if there are 0 bits in the input.
#[inline]
pub fn u64_to_u8(b: u64) -> u8 {
    debug_assert_eq!(popcount64(b), 1);
    bit_scan_forward(b)
}

/// Given a square (u8) that is valid, returns the bitboard representation
/// of that square.
///
/// # Safety
///
/// If the input is greater than 63, an empty u64 will be returned.
#[inline]
pub fn u8_to_u64(s: u8) -> u64 {
    debug_assert!(s < 64);
    (1 as u64).wrapping_shl(s as u32)
}
