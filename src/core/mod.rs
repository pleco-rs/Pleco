#[macro_use]
mod bit_macros;

pub mod bit_twiddles;
pub mod piece_move;
pub mod magic_helper;
pub mod masks;

pub mod sq;
pub mod bitboard;

use std::fmt;

use self::bit_twiddles::*;
use self::masks::*;
use self::sq::SQ;
use self::bitboard::BitBoard;


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

    #[inline]
    pub fn pawn_push(&self) -> i8 {
        match *self {
            Player::White => NORTH,
            Player::Black => SOUTH,
        }
    }

    #[inline]
    pub fn relative_rank_of_sq(&self, sq: SQ) -> Rank {
        self.relative_rank(sq.rank_of_sq())
    }

    #[inline]
    pub fn relative_rank(&self, rank: Rank) -> Rank {
        ALL_RANKS[((rank as u8) ^ (*self as u8 * 7)) as usize]
    }
}


#[inline]
pub fn relative_rank(p: Player, rank: Rank) -> Rank {
    ALL_RANKS[((rank as u8) ^ (p as u8 * 7)) as usize]
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


pub trait PlayerTrait {
    fn player() -> Player;
    fn opp_player() -> Player;

    fn down(sq: SQ) -> SQ;
    fn up(sq: SQ) -> SQ;
    fn left(sq: SQ) -> SQ;
    fn right(sq: SQ) -> SQ;

    fn down_left(sq: SQ) -> SQ;
    fn down_right(sq: SQ) -> SQ;
    fn up_left(sq: SQ) -> SQ;
    fn up_right(sq: SQ) -> SQ;

    fn shift_down(bb: BitBoard) -> BitBoard;
    fn shift_up(bb: BitBoard) -> BitBoard;
    fn shift_left(bb: BitBoard) -> BitBoard;
    fn shift_right(bb: BitBoard) -> BitBoard;

    fn shift_down_left(bb: BitBoard) -> BitBoard;
    fn shift_down_right(bb: BitBoard) -> BitBoard;
    fn shift_up_left(bb: BitBoard) -> BitBoard;
    fn shift_up_right(bb: BitBoard) -> BitBoard;
}

pub struct WhiteType {}
pub struct BlackType {}

impl PlayerTrait for WhiteType {
    fn player() -> Player {
        Player::White
    }
    fn opp_player() -> Player {
        Player::Black
    }

    fn down(sq: SQ) -> SQ { sq - SQ(8) }

    fn up(sq: SQ) -> SQ { sq + SQ(8) }

    fn left(sq: SQ) -> SQ { sq - SQ(1) }

    fn right(sq: SQ) -> SQ { sq + SQ(1) }

    fn down_left(sq: SQ) -> SQ { sq - SQ(9) }

    fn down_right(sq: SQ) -> SQ { sq - SQ(7) }

    fn up_left(sq: SQ) -> SQ { sq + SQ(7) }

    fn up_right(sq: SQ) -> SQ { sq + SQ(9) }

    fn shift_down(bb: BitBoard) -> BitBoard { bb >> 8 }

    fn shift_up(bb: BitBoard) -> BitBoard { bb << 8 }

    fn shift_left(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_A) >> 1 }

    fn shift_right(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_H) << 1 }

    fn shift_down_left(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_A) >> 9 }

    fn shift_down_right(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_H) >> 7 }

    fn shift_up_left(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_A) << 7 }

    fn shift_up_right(bb: BitBoard) -> BitBoard { (bb & !BitBoard::FILE_H) << 9 }
}

impl PlayerTrait for BlackType {
    fn player() -> Player {
        Player::Black
    }

    fn opp_player() -> Player {
        Player::White
    }

    fn down(sq: SQ) -> SQ { sq + SQ(8) }

    fn up(sq: SQ) -> SQ { sq - SQ(8) }

    fn left(sq: SQ) -> SQ { sq + SQ(1) }

    fn right(sq: SQ) -> SQ { sq - SQ(1) }

    fn down_left(sq: SQ) -> SQ { sq + SQ(9) }

    fn down_right(sq: SQ) -> SQ { sq + SQ(7) }

    fn up_left(sq: SQ) -> SQ { sq - SQ(7) }

    fn up_right(sq: SQ) -> SQ { sq - SQ(9) }

    fn shift_down(bb: BitBoard) -> BitBoard { bb << (8) }

    fn shift_up(bb: BitBoard) -> BitBoard { bb >> (8) }

    fn shift_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << (1)
    }

    fn shift_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> (1)
    }

    fn shift_down_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << (9)
    }

    fn shift_down_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) << (7)
    }

    fn shift_up_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) >> (7)
    }

    fn shift_up_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> (9)
    }
}

/// Publicly available move-generation types.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GenTypes {
    All,
    Captures,
    Quiets,
    QuietChecks,
    Evasions,
    NonEvasions
}

pub trait GenTypeTrait {
    fn gen_type() -> GenTypes;
}

pub struct AllGenType {}
pub struct CapturesGenType {}
pub struct QuietsGenType {}
pub struct QuietChecksGenType {}
pub struct EvasionsGenType {}
pub struct NonEvasionsGenType {}

impl GenTypeTrait for AllGenType {
    fn gen_type() -> GenTypes {
        GenTypes::All
    }
}

impl GenTypeTrait for CapturesGenType {
    fn gen_type() -> GenTypes {
        GenTypes::Captures
    }
}

impl GenTypeTrait for QuietsGenType {
    fn gen_type() -> GenTypes {
        GenTypes::Quiets
    }
}

impl GenTypeTrait for QuietChecksGenType {
    fn gen_type() -> GenTypes {
        GenTypes::QuietChecks
    }
}

impl GenTypeTrait for EvasionsGenType {
    fn gen_type() -> GenTypes {
        GenTypes::Evasions
    }
}

impl GenTypeTrait for NonEvasionsGenType {
    fn gen_type() -> GenTypes {
        GenTypes::NonEvasions
    }
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

pub const ALL_PIECES: [Piece; PIECE_CNT] =
    [Piece::P, Piece::N, Piece::B, Piece::R, Piece::Q, Piece::K];

pub trait PieceTrait {
    fn piece_type() -> Piece;
}

pub struct PawnType {}
pub struct KnightType {}
pub struct BishopType {}
pub struct RookType {}
pub struct QueenType {}
pub struct KingType {}

impl PieceTrait for PawnType {
    fn piece_type() -> Piece {
        Piece::P
    }
}

impl PieceTrait for KnightType {
    fn piece_type() -> Piece {
        Piece::N
    }
}

impl PieceTrait for BishopType {
    fn piece_type() -> Piece {
        Piece::B
    }
}

impl PieceTrait for RookType {
    fn piece_type() -> Piece {
        Piece::R
    }
}

impl PieceTrait for QueenType {
    fn piece_type() -> Piece {
        Piece::Q
    }
}

impl PieceTrait for KingType {
    fn piece_type() -> Piece {
        Piece::K
    }
}


pub const ALL_PLAYERS: [Player; 2] = [Player::White, Player::Black];



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


/// Types of Castling available
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CastleType {
    KingSide = 0,
    QueenSide = 1,
}


// For whatever rank the bit is in, gets the whole bitboard
#[inline]
pub fn rank_bb(s: u8) -> u64 {
    RANK_BB[rank_of_sq(s) as usize]
}

#[inline]
pub fn rank_of_sq(s: u8) -> Rank {
    ALL_RANKS[(s >> 3) as usize]
}

#[inline]
pub fn rank_idx_of_sq(s: u8) -> u8 {
    (s >> 3) as u8
}

#[inline]
pub fn file_bb(s: u8) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

#[inline]
pub fn file_of_sq(s: u8) -> File {
    ALL_FILES[(s & 0b0000_0111) as usize]
}

#[inline]
pub fn file_idx_of_sq(s: u8) -> u8 {
    (s & 0b0000_0111) as u8
}

// Assumes only one bit!
#[inline]
pub fn u64_to_u8(b: u64) -> u8 {
    debug_assert_eq!(popcount64(b), 1);
    bit_scan_forward(b)
}

// Given a Square (u8) that is valid, returns the bitboard representaton
#[inline]
pub fn u8_to_u64(s: u8) -> u64 {
    assert!(s < 64);
    (1 as u64).wrapping_shl(s as u32)
}

