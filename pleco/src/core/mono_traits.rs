//! Traits & Dummy Types defined for various Enum types. Shouldn't be used in place
//! of their enum representations.
//!
//! This modules only use is to allow for compile-time mono-morphization of
//! functions / methods, where each method created can be optimized further.

use super::{Player,Piece,GenTypes};
use super::sq::SQ;
use super::bitboard::BitBoard;

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

/// A `PieceTrait` allows for specific functions in relation
/// to the type of move.
///
/// Alike `PlayerTrait` and `GenTypeTrait`, `PieceTrait` is only used for compile-time
/// optimization through mono-morphism. This trait isn't intended to be used
/// elsewhere.
pub trait PieceTrait {
    /// Returns the `Piece` of an object.
    fn piece_type() -> Piece;
}

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
