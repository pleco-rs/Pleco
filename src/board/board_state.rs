use super::castle_rights::Castling;

use core::templates::*;
use core::piece_move::BitMove;
use core::masks::*;

use std::sync::Arc;

/// Holds useful information concerning the current state of the board.
///
/// This is information that is computed upon making a move, and requires expensive computation to do so as well.
/// It is stored in the Heap by 'Board' as an Arc<BoardState>, as cloning the board can lead to multiple
/// references to the same `BoardState`.
///
/// Allows for easy undo-ing of moves as these keep track of their previous board state, forming a
/// Tree-like persistent Stack
#[derive(Clone)]
pub struct BoardState {
    // The Following Fields are easily copied from the previous version and possbily modified
    pub castling: Castling,
    pub rule_50: i16,
    pub ply: u16,
    pub ep_square: SQ,

    // These fields MUST be Recomputed after a move
    pub zobrast: u64,
    pub captured_piece: Option<Piece>,
    pub checkers_bb: BitBoard, // What squares is the current player receiving check from?
    pub blockers_king: [BitBoard; PLAYER_CNT],
    pub pinners_king: [BitBoard; PLAYER_CNT],
    pub check_sqs: [BitBoard; PIECE_CNT],

    pub prev_move: BitMove,

    // Previous State of the board ( one move ago)
    pub prev: Option<Arc<BoardState>>,

    //  castling      ->  Castling Bit Structure, keeping track of if either player can castle.
    //                    as well as if they have castled.
    //  rule50        ->  Moves since last capture, pawn move or castle. Used for Draws.
    //  ply           ->  How many moves deep this current thread is.
    //                    ** NOTE: CURRENTLY UNUSED **
    //  ep_square     ->  If the last move was a double pawn push, this will be equal to the square behind.
    //                    the push. ep_square =  abs(sq_to - sq_from) / 2
    //                    If last move was not a double push, this will equal NO_SQ (which is 64).
    //  zobrast       ->  Zobrist Key of the current board.
    //  capture_piece ->  The Piece (if any) that was last captured
    //  checkers_bb   ->  Bitboard of all pieces who currently check the king

    //  blockers_king ->  Per each player, bitboard of pieces blocking an attack on a that player's king.
    //                    NOTE: Can contain opponents pieces. E.g. a Black Pawn can block an attack of a white king
    //                    if there is a queen (or some other sliding piece) on the same line.
    //  pinners_king  ->  Per each player, bitboard of pieces currently pinning the opponent's king.
    //                    e.g:, a Black Queen pinning a piece (of either side) to White's King
    //  check_sqs     ->  Array of BitBoards where for Each Piece, gives a spot the piece can move to where
    //                    the opposing player's king would be in check.
}

impl BoardState {
    /// Constructs a `BoardState` from the starting position
    pub fn default() -> BoardState {
        BoardState {
            castling: Castling::all(),
            rule_50: 0,
            ply: 0,
            ep_square: NO_SQ,
            zobrast: 0,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
            prev_move: BitMove::null(),
            prev: None,
        }
    }

    /// Constructs a blank `BoardState`.
    pub fn blank() -> BoardState {
        BoardState {
            castling: Castling::empty(),
            rule_50: 0,
            ply: 0,
            ep_square: NO_SQ,
            zobrast: 0,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
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
            zobrast: self.zobrast,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
            prev_move: BitMove::null(),
            prev: self.get_prev(),
        }
    }

    /// Return the previous BoardState from one move ago.
    pub fn get_prev(&self) -> Option<Arc<BoardState>> {
        (&self).prev.as_ref().cloned()
    }


    pub fn backtrace(&self) {
        self.print_info();
        if self.prev.is_some() {
            self.get_prev().unwrap().backtrace();
        }
    }

    pub fn print_info(&self) {

        print!("ply: {}, move played: {} ",self.ply, self.prev_move);
        if self.captured_piece.is_some() {
            print!("cap {}", self.captured_piece.unwrap());
        }
        if self.checkers_bb != 0 {
            print!("in check {}", bb_to_sq(self.checkers_bb));
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
