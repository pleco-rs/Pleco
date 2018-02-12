use super::psqt;
use super::zobrist;
use super::magic;
use super::boards;

use {SQ,BitBoard,Player,PieceType,File,Rank};
use core::score::{Score,Value};

use std::sync::atomic::{AtomicBool,Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init_statics() {
    if !INITIALIZED.load(Ordering::AcqRel) {
        zobrist::init_zobrist();
        psqt::init_psqt();
        magic::init_magics();
        boards::init_boards();
        INITIALIZED.store(true, Ordering::AcqRel);
    }
}

// MAGIC FUNCTIONS

#[inline(always)]
pub fn bishop_moves(occupied: BitBoard, sq: SQ) -> BitBoard {
    debug_assert!(sq.is_okay());
    BitBoard(magic::bishop_attacks(occupied.0, sq.0))
}

#[inline(always)]
pub fn rook_moves(occupied: BitBoard, sq: SQ) -> BitBoard {
    debug_assert!(sq.is_okay());
    BitBoard(magic::rook_attacks(occupied.0, sq.0))
}

#[inline(always)]
pub fn queen_moves(occupied: BitBoard, sq: SQ) -> BitBoard {
    debug_assert!(sq.is_okay());
    BitBoard(magic::rook_attacks(occupied.0, sq.0)
           | magic::bishop_attacks(occupied.0, sq.0))
}

// BOARD FUNCTIONS

#[inline(always)]
pub fn knight_moves(sq: SQ) -> BitBoard {
    BitBoard(boards::knight_moves(sq))
}

#[inline(always)]
pub fn king_moves(sq: SQ) -> BitBoard {
    BitBoard(boards::king_moves(sq))
}

/// Get the distance of two squares.
#[inline(always)]
pub fn distance_of_sqs(sq_one: SQ, sq_two: SQ) -> u8 {
    boards::distance_of_sqs(sq_one, sq_two)
}

/// Get the line (diagonal / file / rank) `BitBoard` that two squares both exist on, if it exists.
#[inline(always)]
pub fn line_bb(sq_one: SQ, sq_two: SQ) -> BitBoard {
    BitBoard(boards::line_bb(sq_one, sq_two))
}

/// Get the line (diagonal / file / rank) `BitBoard` between two squares, not including the squares, if it exists.
#[inline(always)]
pub fn between_bb(sq_one: SQ, sq_two: SQ) -> BitBoard {
    BitBoard(boards::between_bb(sq_one, sq_two))
}

/// Gets the adjacent files `BitBoard` of the square
#[inline(always)]
pub fn adjacent_sq_file(sq: SQ) -> BitBoard {
    BitBoard(boards::adjacent_sq_file(sq))
}

/// Gets the adjacent files `BitBoard` of the file
#[inline(always)]
pub fn adjacent_file(f: File) -> BitBoard {
    BitBoard(boards::adjacent_file(f))
}

/// Pawn attacks `BitBoard` from a given square, per player.
/// Basically, given square x, returns the BitBoard of squares a pawn on x attacks.
#[inline(always)]
pub fn pawn_attacks_from(sq: SQ, player: Player) -> BitBoard {
    BitBoard(boards::pawn_attacks_from(sq,player))
}

/// Returns if three Squares are in the same diagonal, file, or rank.
#[inline(always)]
pub fn aligned(s1: SQ, s2: SQ, s3: SQ) -> bool {
    boards::aligned(s1,s2,s3)
}

/// Returns the ring of bits surrounding the square sq at a specified distance.
///
/// # Safety
///
/// distance must be less than 8, or else a panic will occur.
#[inline(always)]
pub fn ring_distance(sq: SQ, distance: u8) -> BitBoard {
    BitBoard(boards::ring_distance(sq,distance))
}

/// Returns the BitBoard of all squares in the rank in front of the given one.
#[inline(always)]
pub fn forward_rank_bb(player: Player, rank: Rank) -> BitBoard {
    BitBoard(boards::forward_rank_bb(player,rank))
}

/// Returns the `BitBoard` of all squares that can be attacked by a pawn
/// of the same color when it moves along its file, starting from the
/// given square. Basically, if the pawn progresses along the same file
/// for the entire game, this bitboard would contain all possible forward squares
/// it could attack
///
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
#[inline(always)]
pub fn pawn_attacks_span(player: Player, sq: SQ) -> BitBoard {
    BitBoard(boards::pawn_attacks_span(player,sq))
}

/// Returns the BitBoard of all squares in the file in front of the given one.
///
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
#[inline(always)]
pub fn forward_file_bb(player: Player, sq: SQ) -> BitBoard {
    BitBoard(boards::forward_file_bb(player,sq))
}


/// Returns a `BitBoard` allowing for testing of the a pawn being a
/// "passed pawn".
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behvaior may occur.
#[inline(always)]
pub fn passed_pawn_mask(player: Player, sq: SQ) -> BitBoard {
    BitBoard(boards::passed_pawn_mask(player, sq))
}

// ZOBRIST FUNCTIONS

#[inline(always)]
pub fn z_square(sq: SQ, player: Player, piece: PieceType) -> u64 {
    zobrist::z_square(sq, player, piece)
}

#[inline(always)]
pub fn z_ep(sq: SQ) -> u64 {
    zobrist::z_ep(sq)
}

#[inline(always)]
pub fn z_castle(castle: u8) -> u64 {
    zobrist::z_castle(castle)
}

#[inline(always)]
pub fn z_side() -> u64 {
    zobrist::z_side()
}


// PSQT FUNCTIONS

/// Returns the score for a player's piece being at a particular square.
#[inline(always)]
pub fn psq(piece: PieceType, player: Player, sq: SQ) -> Score {
    psqt::psq(piece, player, sq)
}

/// Returns the value of a piece for a player. If `eg` is true, it returns the end game value. Otherwise,
/// it'll return the midgame value.
#[inline(always)]
pub fn piece_value(piece: PieceType, eg: bool) -> Value {
    psqt::piece_value(piece, eg)
}