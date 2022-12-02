//! Traits and Dummy Types defined for various Enum types. Shouldn't be used in place
//! of their enum representations.
//!
//! This modules only use is to allow for compile-time mono-morphization of
//! functions / methods, where each method created can be optimized further.
//!
//! We are awaiting the stabilization of `const fn` and constant generics to remove these traits.

use super::bitboard::BitBoard;
use super::sq::SQ;
use super::{GenTypes, PieceType, Player};

/// Defines a Player Trait, allowing for specific functions in relation
/// to a certain player.
///
/// These shouldn't be used in place of `Player`, as they are only used for
/// compile-time optimizations of certain functions.
pub trait PlayerTrait {
    /// Return the current `Player`.
    fn player() -> Player;

    /// Return the opposing `Player`.
    fn opp_player() -> Player;

    /// Returns the index of the player
    fn player_idx() -> usize;

    /// Given a `SQ`, return a square that is down relative to the current player.
    fn down(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is up relative to the current player.
    fn up(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is left relative to the current player.
    fn left(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is right relative to the current player.
    fn right(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is down-left relative to the current player.
    fn down_left(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is down-right relative to the current player.
    fn down_right(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is up-right relative to the current player.
    fn up_left(sq: SQ) -> SQ;

    /// Given a `SQ`, return a square that is up-right relative to the current player.
    fn up_right(sq: SQ) -> SQ;

    /// Return the same BitBoard shifted "down" relative to the current player.
    fn shift_down(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "up" relative to the current player.
    fn shift_up(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "left" relative to the current player. Does not
    /// include the left-most file in the result.
    fn shift_left(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "right" relative to the current player. Does not
    /// include the left-most file in the result.
    fn shift_right(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "left" and "down" relative to the current player.
    /// Does not include the left-most file in the result.
    fn shift_down_left(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "right" and "down" relative to the current player.
    /// Does not include the right-most file in the result.
    fn shift_down_right(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "left" and "up" relative to the current player.
    /// Does not include the left-most file in the result.
    fn shift_up_left(bb: BitBoard) -> BitBoard;

    /// Return the same BitBoard shifted "right" and "up" relative to the current player.
    /// Does not include the right-most file in the result.
    fn shift_up_right(bb: BitBoard) -> BitBoard;
}

/// Dummy type to represent a `Player::White` which implements `PlayerTrait`.
pub struct WhiteType {}

/// Dummy type to represent a `Player::Black` which implements `PlayerTrait`.
pub struct BlackType {}

impl PlayerTrait for WhiteType {
    #[inline(always)]
    fn player() -> Player {
        Player::White
    }
    #[inline(always)]
    fn opp_player() -> Player {
        Player::Black
    }

    #[inline(always)]
    fn player_idx() -> usize {
        Player::White as usize
    }

    #[inline(always)]
    fn down(sq: SQ) -> SQ {
        sq - SQ(8)
    }

    #[inline(always)]
    fn up(sq: SQ) -> SQ {
        sq + SQ(8)
    }

    #[inline(always)]
    fn left(sq: SQ) -> SQ {
        sq - SQ(1)
    }

    #[inline(always)]
    fn right(sq: SQ) -> SQ {
        sq + SQ(1)
    }

    #[inline(always)]
    fn down_left(sq: SQ) -> SQ {
        sq - SQ(9)
    }

    #[inline(always)]
    fn down_right(sq: SQ) -> SQ {
        sq - SQ(7)
    }

    #[inline(always)]
    fn up_left(sq: SQ) -> SQ {
        sq + SQ(7)
    }

    #[inline(always)]
    fn up_right(sq: SQ) -> SQ {
        sq + SQ(9)
    }

    #[inline(always)]
    fn shift_down(bb: BitBoard) -> BitBoard {
        bb >> 8
    }

    #[inline(always)]
    fn shift_up(bb: BitBoard) -> BitBoard {
        bb << 8
    }

    #[inline(always)]
    fn shift_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> 1
    }

    #[inline(always)]
    fn shift_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << 1
    }

    #[inline(always)]
    fn shift_down_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> 9
    }

    #[inline(always)]
    fn shift_down_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) >> 7
    }

    #[inline(always)]
    fn shift_up_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) << 7
    }

    #[inline(always)]
    fn shift_up_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << 9
    }
}

impl PlayerTrait for BlackType {
    #[inline(always)]
    fn player() -> Player {
        Player::Black
    }

    #[inline(always)]
    fn opp_player() -> Player {
        Player::White
    }

    #[inline(always)]
    fn player_idx() -> usize {
        Player::Black as usize
    }

    #[inline(always)]
    fn down(sq: SQ) -> SQ {
        sq + SQ(8)
    }

    #[inline(always)]
    fn up(sq: SQ) -> SQ {
        sq - SQ(8)
    }

    #[inline(always)]
    fn left(sq: SQ) -> SQ {
        sq + SQ(1)
    }

    #[inline(always)]
    fn right(sq: SQ) -> SQ {
        sq - SQ(1)
    }

    #[inline(always)]
    fn down_left(sq: SQ) -> SQ {
        sq + SQ(9)
    }

    #[inline(always)]
    fn down_right(sq: SQ) -> SQ {
        sq + SQ(7)
    }

    #[inline(always)]
    fn up_left(sq: SQ) -> SQ {
        sq - SQ(7)
    }

    #[inline(always)]
    fn up_right(sq: SQ) -> SQ {
        sq - SQ(9)
    }

    #[inline(always)]
    fn shift_down(bb: BitBoard) -> BitBoard {
        bb << (8)
    }

    #[inline(always)]
    fn shift_up(bb: BitBoard) -> BitBoard {
        bb >> (8)
    }

    #[inline(always)]
    fn shift_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << (1)
    }

    #[inline(always)]
    fn shift_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> (1)
    }

    #[inline(always)]
    fn shift_down_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) << (9)
    }

    #[inline(always)]
    fn shift_down_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) << (7)
    }

    #[inline(always)]
    fn shift_up_left(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_H) >> (7)
    }

    #[inline(always)]
    fn shift_up_right(bb: BitBoard) -> BitBoard {
        (bb & !BitBoard::FILE_A) >> (9)
    }
}

/// A `GenTypeTrait` allows for specific functions in relation
/// to a certain type of move generation.
///
/// Alike `PlayerTrait`, `GenTypeTrait` is only used for compile-time
/// optimization through mono-morphism. This trait isn't intended to be used
/// elsewhere.
pub trait GenTypeTrait {
    /// Returns the `GenType`.
    fn gen_type() -> GenTypes;
}

/// Dummy type to represent a `GenTypes::All` which implements `GenTypeTrait`.
pub struct AllGenType {}
/// Dummy type to represent a `GenTypes::Captures` which implements `GenTypeTrait`.
pub struct CapturesGenType {}
/// Dummy type to represent a `GenTypes::Quiets` which implements `GenTypeTrait`.
pub struct QuietsGenType {}
/// Dummy type to represent a `GenTypes::QuietChecks` which implements `GenTypeTrait`.
pub struct QuietChecksGenType {}
/// Dummy type to represent a `GenTypes::Evasions` which implements `GenTypeTrait`.
pub struct EvasionsGenType {}
/// Dummy type to represent a `GenTypes::NonEvasions` which implements `GenTypeTrait`.
pub struct NonEvasionsGenType {}

impl GenTypeTrait for AllGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::All
    }
}

impl GenTypeTrait for CapturesGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::Captures
    }
}

impl GenTypeTrait for QuietsGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::Quiets
    }
}

impl GenTypeTrait for QuietChecksGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::QuietChecks
    }
}

impl GenTypeTrait for EvasionsGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::Evasions
    }
}

impl GenTypeTrait for NonEvasionsGenType {
    #[inline(always)]
    fn gen_type() -> GenTypes {
        GenTypes::NonEvasions
    }
}

/// A `PieceTrait` allows for specific functions in relation
/// to the type of move.
///
/// Alike `PlayerTrait` and `GenTypeTrait`, `PieceTrait` is only used for compile-time
/// optimization through mono-morphism. This trait isn't intended to be used
/// elsewhere.
pub trait PieceTrait {
    /// Returns the `Piece` of an object.
    fn piece_type() -> PieceType;
}

// Returns the next `PieceTrait` for the PieceTrait.
//
// Pawn   -> KnightType
// Knight -> BishopType
// Bishop -> RookType
// Rook   -> QueenType
// Queen  -> KingType
// King   -> KingType
//pub(crate) fn incr_pt<P: PieceTrait>(p: P) -> impl PieceTrait {
//    match <P as PieceTrait>::piece_type() {
//        PieceType::P => PawnType{},
//        PieceType::N => KnightType{},
//        PieceType::B => BishopType{},
//        PieceType::R => QueenType{},
//        PieceType::Q => KingType{},
//        PieceType::K => KingType{},
//    }
//}

/// Dummy type to represent a `Piece::P` which implements `PieceTrait`.
pub struct PawnType {}
/// Dummy type to represent a `Piece::N` which implements `PieceTrait`.
pub struct KnightType {}
/// Dummy type to represent a `Piece::B` which implements `PieceTrait`.
pub struct BishopType {}
/// Dummy type to represent a `Piece::R` which implements `PieceTrait`.
pub struct RookType {}
/// Dummy type to represent a `Piece::Q` which implements `PieceTrait`.
pub struct QueenType {}
/// Dummy type to represent a `Piece::K` which implements `PieceTrait`.
pub struct KingType {}

impl PieceTrait for PawnType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::P
    }
}

impl PawnType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        KnightType {}
    }
}

impl PieceTrait for KnightType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::N
    }
}

impl KnightType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        BishopType {}
    }
}

impl PieceTrait for BishopType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::B
    }
}

impl BishopType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        RookType {}
    }
}

impl PieceTrait for RookType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::R
    }
}

impl RookType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        QueenType {}
    }
}

impl PieceTrait for QueenType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::Q
    }
}

impl QueenType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        KingType {}
    }
}

impl PieceTrait for KingType {
    #[inline(always)]
    fn piece_type() -> PieceType {
        PieceType::K
    }
}

impl KingType {
    #[inline(always)]
    fn incr() -> impl PieceTrait {
        KingType {}
    }
}
