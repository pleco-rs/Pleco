//! Statically initialized lookup tables.
//!
//! Whenever a `Board` is created, these are also created as well. Calling `Helper::new()` will
//! initialize the tables the first time it's called, and successive calls won't waste time
//! initializing the table.
//!
//! It is highly recommended to go through a `Helper` to access these tables, as the access will
//! guarantee that the tables are initialized in the first place.
//!
//! If you want the same functions, but can ensure the Tables are initialized, see [`helper::prelude`]
//! for those raw functions.
//!
//! [`helper::prelude`]: prelude/index.html

mod boards;
mod magic;
pub mod prelude;
mod psqt;
mod zobrist;

use core::score::{Score, Value};
use {BitBoard, File, Piece, Player, Rank, SQ};

/// Helper structure for accessing statically-initialized tables and other constants.
///
/// Guarantees that the tables will be initialized upon access through a `Helper`.
#[derive(Copy, Clone)]
pub struct Helper {}

unsafe impl Send for Helper {}

unsafe impl Sync for Helper {}

impl Default for Helper {
    fn default() -> Self {
        Helper::new()
    }
}

impl Helper {
    /// Creates a new `Helper` Object, automatically initializing all the needed tables.
    ///
    /// Calling this method multiple times does not waste time computing the static variables if
    /// already initialized. [`init_statics`] also does the same thing as well.
    ///
    /// [`init_statics`]: prelude/fn.init_statics.html
    pub fn new() -> Self {
        prelude::init_statics();
        Helper {}
    }

    /// Generate Bishop Moves `BitBoard` from a bishop square and all occupied squares on the board.
    /// This function will return captures to pieces on both sides. The resulting `BitBoard` must be
    /// AND'd with the inverse of the intending moving player's pieces.
    #[inline(always)]
    pub fn bishop_moves(self, occupied: BitBoard, sq: SQ) -> BitBoard {
        prelude::bishop_moves(occupied, sq)
    }

    /// Generate Rook Moves `BitBoard` from a bishop square and all occupied squares on the board.
    /// This function will return captures to pieces on both sides. The resulting `BitBoard` must be
    /// AND'd with the inverse of the intending moving player's pieces.
    #[inline(always)]
    pub fn rook_moves(self, occupied: BitBoard, sq: SQ) -> BitBoard {
        prelude::rook_moves(occupied, sq)
    }

    /// Generate Queen Moves `BitBoard` from a bishop square and all occupied squares on the board.
    /// This function will return captures to pieces on both sides. The resulting `BitBoard` must be
    /// AND'd with the inverse of the intending moving player's pieces.
    #[inline(always)]
    pub fn queen_moves(self, occupied: BitBoard, sq: SQ) -> BitBoard {
        prelude::queen_moves(occupied, sq)
    }

    /// Generate Knight Moves `BitBoard` from a source square.
    #[inline(always)]
    pub fn knight_moves(self, sq: SQ) -> BitBoard {
        prelude::knight_moves(sq)
    }

    /// Generate King moves `BitBoard` from a source square.
    #[inline(always)]
    pub fn king_moves(self, sq: SQ) -> BitBoard {
        prelude::king_moves(sq)
    }

    /// Get the distance of two squares.
    #[inline(always)]
    pub fn distance_of_sqs(self, sq_one: SQ, sq_two: SQ) -> u8 {
        prelude::distance_of_sqs(sq_one, sq_two)
    }

    /// Get the line (diagonal / file / rank) `BitBoard` that two squares both exist on, if it exists.
    #[inline(always)]
    pub fn line_bb(self, sq_one: SQ, sq_two: SQ) -> BitBoard {
        prelude::line_bb(sq_one, sq_two)
    }

    /// Get the line (diagonal / file / rank) `BitBoard` between two squares, not including the squares, if it exists.
    #[inline(always)]
    pub fn between_bb(self, sq_one: SQ, sq_two: SQ) -> BitBoard {
        prelude::between_bb(sq_one, sq_two)
    }

    /// Gets the adjacent files `BitBoard` of the square
    #[inline(always)]
    pub fn adjacent_sq_file(self, sq: SQ) -> BitBoard {
        prelude::adjacent_sq_file(sq)
    }

    /// Gets the adjacent files `BitBoard` of the file
    #[inline(always)]
    pub fn adjacent_file(self, f: File) -> BitBoard {
        prelude::adjacent_file(f)
    }

    /// Pawn attacks `BitBoard` from a given square, per player.
    /// Basically, given square x, returns the BitBoard of squares a pawn on x attacks.
    #[inline(always)]
    pub fn pawn_attacks_from(self, sq: SQ, player: Player) -> BitBoard {
        prelude::pawn_attacks_from(sq, player)
    }

    /// Returns if three Squares are in the same diagonal, file, or rank.
    #[inline(always)]
    pub fn aligned(self, s1: SQ, s2: SQ, s3: SQ) -> bool {
        prelude::aligned(s1, s2, s3)
    }

    /// Returns the ring of bits surrounding the square sq at a specified distance.
    ///
    /// # Safety
    ///
    /// distance must be less than 8, or else a panic will occur.
    #[inline(always)]
    pub fn ring_distance(self, sq: SQ, distance: u8) -> BitBoard {
        prelude::ring_distance(sq, distance)
    }

    /// Returns the BitBoard of all squares in the rank in front of the given one.
    #[inline(always)]
    pub fn forward_rank_bb(self, player: Player, rank: Rank) -> BitBoard {
        prelude::forward_rank_bb(player, rank)
    }

    /// Returns the `BitBoard` of all squares that can be attacked by a pawn
    /// of the same color when it moves along its file, starting from the
    /// given square. Basically, if the pawn progresses along the same file
    /// for the entire game, this bitboard would contain all possible forward squares
    /// it could attack
    ///
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
    #[inline(always)]
    pub fn pawn_attacks_span(self, player: Player, sq: SQ) -> BitBoard {
        prelude::pawn_attacks_span(player, sq)
    }

    /// Returns the BitBoard of all squares in the file in front of the given one.
    ///
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
    #[inline(always)]
    pub fn forward_file_bb(self, player: Player, sq: SQ) -> BitBoard {
        prelude::forward_file_bb(player, sq)
    }

    /// Returns a `BitBoard` allowing for testing of the a pawn being a
    /// "passed pawn".
    ///
    /// # Safety
    ///
    /// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
    #[inline(always)]
    pub fn passed_pawn_mask(self, player: Player, sq: SQ) -> BitBoard {
        prelude::passed_pawn_mask(player, sq)
    }

    /// Returns the zobrist hash of a specific player's piece being at a particular square.
    #[inline(always)]
    pub fn z_square(self, sq: SQ, piece: Piece) -> u64 {
        prelude::z_square(sq, piece)
    }

    /// Returns the zobrist hash of a given file having an en-passant square.
    #[inline(always)]
    pub fn z_ep(self, sq: SQ) -> u64 {
        prelude::z_ep(sq)
    }

    /// Returns a zobrist hash of the castling rights, as defined by the Board.
    #[inline(always)]
    pub fn z_castle(self, castle: u8) -> u64 {
        prelude::z_castle(castle)
    }

    /// Returns Zobrist Hash of flipping sides.
    #[inline(always)]
    pub fn z_side(self) -> u64 {
        prelude::z_side()
    }

    /// Returns the score for a player's piece being at a particular square.
    #[inline(always)]
    pub fn psq(self, piece: Piece, sq: SQ) -> Score {
        prelude::psq(piece, sq)
    }

    /// Returns the value of a piece for a player. If `eg` is true, it returns the end game value. Otherwise,
    /// it'll return the midgame value.
    #[inline(always)]
    pub fn piece_value(self, piece: Piece, eg: bool) -> Value {
        prelude::piece_value(piece, eg)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_helper() {
        Helper::new();
    }
}
