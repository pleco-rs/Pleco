//! Contains various components and structures supporting the creation of a chessboard. This
//! includes `SQ`, `BitBoard`, `Player`, `Piece`, `GenTypes`, `Rank`, and `File`. Also holds
//! the statically created `MagicHelper`, which at runtime creates various lookup tables.

#[macro_use]
mod bit_macros;

pub mod bit_twiddles;
pub mod piece_move;
pub mod magic_helper;
pub mod masks;
pub mod mono_traits;
pub mod sq;
pub mod bitboard;

use self::bit_twiddles::*;
use self::masks::*;
use self::sq::SQ;

use std::fmt;


/// Array of all possible pieces, indexed by their enum value.
pub const ALL_PIECES: [Piece; PIECE_CNT] =
    [Piece::P, Piece::N, Piece::B, Piece::R, Piece::Q, Piece::K];


/// Array of both players, indexed by their enum value.
pub const ALL_PLAYERS: [Player; 2] = [Player::White, Player::Black];

/// Array of all `Files`s, indexed by their enum value.
pub const ALL_FILES: [File; FILE_CNT] = [
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
pub const ALL_RANKS: [Rank; RANK_CNT] = [
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
    #[inline]
    pub fn other_player(&self) -> Player {
        match *self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    /// Returns the relative square from a given square.
    #[inline]
    pub fn relative_square(&self, sq: SQ) -> SQ {
        assert!(sq.is_okay());
        sq ^ SQ((*self) as u8 * 56)
    }

    /// Gets the direction of a pawn push for a given player.
    #[inline]
    pub fn pawn_push(&self) -> i8 {
        match *self {
            Player::White => NORTH,
            Player::Black => SOUTH,
        }
    }

    /// Returns the relative rank of a square in relation to a player.
    ///
    ///  # Examples
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
    #[inline]
    pub fn relative_rank_of_sq(&self, sq: SQ) -> Rank {
        self.relative_rank(sq.rank_of_sq())
    }

    /// Returns the relative rank of a rank in relation to a player.
    ///
    ///  # Examples
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
    #[inline]
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

/// Different types of move generating options.
///
///`GenTypes::All` -> All available moves.
///`GenTypes::Captures` -> All captures and both capture/non-capture promotions.
///`GenTypes::Quiets` -> All non captures and both capture/non-capture promotions.
///`GenTypes::QuietChecks` -> Moves likely to give check.
///`GenTypes::Evasions` -> Generates evasions for a board in check.
///`GenTypes::NonEvasions` -> Generates all moves for a board not in check.
///
///  # Safety
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



/// Enum for all the possible Pieces.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    K = 5,
    Q = 4,
    R = 3,
    B = 2,
    N = 1,
    P = 0,
}

impl Piece {
    /// Returns the relative value of a piece.
    ///
    /// Used for sorting moves.
    #[inline]
    pub fn value(&self) -> i8 {
        match *self {
            Piece::P => 1,
            Piece::N | Piece::B => 3,
            Piece::R => 5,
            Piece::Q => 8,
            Piece::K => 0,
        }
    }

    /// Return the lowercase character of a `Piece`.
    #[inline]
    pub fn char_lower(&self) -> char {
        match *self {
            Piece::P => 'p',
            Piece::N => 'n',
            Piece::B => 'b',
            Piece::R => 'r',
            Piece::Q => 'q',
            Piece::K => 'k',
        }
    }

    /// Return the uppercase character of a `Piece`.
    #[inline]
    pub fn char_upper(&self) -> char {
        match *self {
            Piece::P => 'P',
            Piece::N => 'N',
            Piece::B => 'B',
            Piece::R => 'R',
            Piece::Q => 'Q',
            Piece::K => 'K',
        }
    }

}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Piece::P => "Pawn",
            Piece::N => "Knight",
            Piece::B => "Bishop",
            Piece::R => "Rook",
            Piece::Q => "Queen",
            Piece::K => "King"
        };
        f.pad(s)
    }
}

/// Enum for the Files of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
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

/// Enum for the Ranks of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
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
pub enum CastleType {
    KingSide = 0,
    QueenSide = 1,
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding rank as a u64.
#[inline]
pub fn rank_bb(s: u8) -> u64 {
    RANK_BB[rank_of_sq(s) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank`.
#[inline]
pub fn rank_of_sq(s: u8) -> Rank {
    ALL_RANKS[(s >> 3) as usize]
}

/// For whatever rank the bit (inner value of a `SQ`) is, returns the
/// corresponding `Rank` index.
#[inline]
pub fn rank_idx_of_sq(s: u8) -> u8 {
    (s >> 3) as u8
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding file as a u64.
#[inline]
pub fn file_bb(s: u8) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File`.
#[inline]
pub fn file_of_sq(s: u8) -> File {
    ALL_FILES[(s & 0b0000_0111) as usize]
}

/// For whatever file the bit (inner value of a `SQ`) is, returns the
/// corresponding `File` index.
#[inline]
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

