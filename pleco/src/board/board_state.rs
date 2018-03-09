//! Contains the `BoardState` structure for the `Board`. Helps to preserve the previous state
//! of the board without needing to re-compute information.
//!
//! As the [`BoardState`] is automatically created for each position of the [`Board`], there is
//! little need for interacting directly with this module.
//!
//! See [this blog post](https://sfleischman105.github.io/2017/10/26/creating-a-chess-engine.html) for
//! more information about the design of the [`BoardState`].
//!
//! [`BoardState`]: struct.BoardState.html
//! [`Board`]: ../struct.Board.html

use super::castle_rights::Castling;

use core::*;
use core::piece_move::BitMove;
use core::sq::{SQ,NO_SQ};
use core::bitboard::BitBoard;
use core::masks::*;
use core::score::{Value,Score};

//use std::sync::Arc;
use tools::pleco_arc::Arc;

/// Holds useful information concerning the current state of the [`Board`].
///
/// This is information that is computed upon making a move, and requires expensive computation to do so as well.
/// It is stored in the Heap by [`Board`] as an `Arc<BoardState>`, as cloning the board can lead to multiple
/// references to the same `BoardState`.
///
/// Allows for easy undo-ing of moves as these keep track of their previous board state, forming a
/// Tree-like persistent Stack.
///
/// [`Board`]: ../struct.Board.html
#[derive(Clone)]
pub struct BoardState {
    // The Following Fields are easily copied from the previous version and possibly modified
    /// The castling rights for the current board.
    pub castling: Castling,
    /// Rule 50 for the current board. Tracks the moves since a capture, pawn move, or castle.
    pub rule_50: i16,
    /// Returns how many plies deep the current Board is. In simpler terms, how many moves have been played since
    /// the `Board` was created.
    pub ply: u16,
    /// If the last move was a double pawn push, this will be equal to the square behind.
    /// the push. So, `ep_square = abs(sq_to - sq_from) / 2`. If the last move was not
    /// a double pawn push, then `ep_square = NO_SQ`.
    pub ep_square: SQ,

    /// The positional score of the board.
    pub psq: Score,

    // These fields MUST be Recomputed after a move

    /// The Zobrist key of the board.
    pub zobrast: u64,
    /// The Hash key of the current pawn configuration.
    pub pawn_key: u64,
    /// The Hash key of the current material configuration.
    pub material_key: u64,
    /// The value of each player's non-pawn pieces.
    pub nonpawn_material: [Value; PLAYER_CNT],
    /// The last captured Piece, if any.
    pub captured_piece: Option<PieceType>,
    /// A `BitBoard` of the current pieces giving check.
    pub checkers_bb: BitBoard,
    /// Per each player, `BitBoard` of pieces blocking an attack on a that player's king.
    /// This field can contain opponents pieces. E.g. a Black Pawn can block an attack of a white king
    /// if there is a queen (or some other sliding piece) on the same line.
    pub blockers_king: [BitBoard; PLAYER_CNT],
    /// Per each player, `BitBoard` of pieces currently pinning the opponent's king.
    //  e.g:, a Black Queen pinning a piece (of either side) to White's King
    pub pinners_king: [BitBoard; PLAYER_CNT],
    /// Array of BitBoards where for Each Piece, gives a spot the piece can move to where
    /// the opposing player's king would be in check.
    pub check_sqs: [BitBoard; PIECE_TYPE_CNT],
    /// returns the previous move, if any, that was played. Returns `BitMove::NULL` if there was no
    /// previous move played.
    pub prev_move: BitMove,
    /// Previous State of the board (from one move ago).
    pub prev: Option<Arc<BoardState>>,
}

impl BoardState {
    /// Constructs a `BoardState` from the starting position.
    pub const fn default() -> BoardState {
        BoardState {
            castling: Castling::all_castling(),
            rule_50: 0,
            ply: 0,
            ep_square: NO_SQ,
            psq: Score::ZERO,
            zobrast: 0,
            pawn_key: 0,
            material_key: 0,
            nonpawn_material: [0; PLAYER_CNT],
            captured_piece: None,
            checkers_bb: BitBoard(0),
            blockers_king: [BitBoard(0); PLAYER_CNT],
            pinners_king: [BitBoard(0); PLAYER_CNT],
            check_sqs: [BitBoard(0); PIECE_TYPE_CNT],
            prev_move: BitMove::null(),
            prev: None,
        }
    }

    /// Constructs a blank `BoardState`.
    pub const fn blank() -> BoardState {
        BoardState {
            castling: Castling::empty_set(),
            rule_50: 0,
            ply: 0,
            ep_square: NO_SQ,
            psq: Score::ZERO,
            zobrast: 0,
            pawn_key: 0,
            material_key: 0,
            nonpawn_material: [0; PLAYER_CNT],
            captured_piece: None,
            checkers_bb: BitBoard(0),
            blockers_king: [BitBoard(0); PLAYER_CNT],
            pinners_king: [BitBoard(0); PLAYER_CNT],
            check_sqs: [BitBoard(0); PIECE_TYPE_CNT],
            prev_move: BitMove::null(),
            prev: None,
        }
    }

    /// Constructs a partial clone of a `BoardState`.
    ///
    /// Castling, rule_50, ply, and ep_square are copied. The copied fields need to be
    /// modified accordingly, and the remaining fields need to be generated.
    pub fn partial_clone(&self) -> BoardState {
        BoardState {
            castling: self.castling,
            rule_50: self.rule_50,
            ply: self.ply,
            ep_square: self.ep_square,
            psq: self.psq,
            zobrast: self.zobrast,
            pawn_key: self.pawn_key,
            material_key: self.material_key,
            nonpawn_material: self.nonpawn_material,
            captured_piece: None,
            checkers_bb: BitBoard(0),
            blockers_king: [BitBoard(0); PLAYER_CNT],
            pinners_king: [BitBoard(0); PLAYER_CNT],
            check_sqs: [BitBoard(0); PIECE_TYPE_CNT],
            prev_move: BitMove::null(),
            prev: self.get_prev(),
        }
    }

    /// Return the previous BoardState from one move ago.
    #[inline]
    pub fn get_prev(&self) -> Option<Arc<BoardState>> {
        (&self).prev.as_ref().cloned()
    }

    /// Iterates through all previous `BoardStates` and prints their information.
    ///
    /// Used primarily for debugging.
    pub fn backtrace(&self) {
        self.print_info();
        if let Some(ref prev) = self.prev {
            prev.backtrace();
        }
    }

    /// Prints information about the current `BoardState`.
    pub fn print_info(&self) {
        print!("ply: {}, move played: {} ",self.ply, self.prev_move);
        if let Some(piece) = self.captured_piece {
            print!("cap {}", piece);
        }
        if !self.checkers_bb.is_empty() {
            print!("in check {}", self.checkers_bb.to_sq());
        }
        println!();
    }
}

impl PartialEq for BoardState {
    fn eq(&self, other: &BoardState) -> bool {
        self.castling == other.castling &&
            self.rule_50 == other.rule_50 &&
            self.ep_square == other.ep_square &&
            self.zobrast == other.zobrast &&
            self.captured_piece == other.captured_piece &&
            self.checkers_bb == other.checkers_bb &&
            self.blockers_king == other.blockers_king &&
            self.pinners_king == other.pinners_king &&
            self.check_sqs == other.check_sqs
    }
}
