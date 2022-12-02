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
use super::Board;

use core::bitboard::BitBoard;
use core::masks::*;
use core::piece_move::BitMove;
use core::score::{Score, Value};
use core::sq::{NO_SQ, SQ};
use core::*;

use helper::prelude::*;
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
    pub zobrist: u64,
    /// The Hash key of the current pawn configuration.
    pub pawn_key: u64,
    /// The Hash key of the current material configuration.
    pub material_key: u64,
    /// The value of each player's non-pawn pieces.
    pub nonpawn_material: [Value; PLAYER_CNT],
    /// The last captured Piece, if any.
    pub captured_piece: PieceType,
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
    /// The previous move, if any, that was played. Returns `BitMove::NULL` if there was no
    /// previous move played.
    pub prev_move: BitMove,
    /// Previous State of the board (from one move ago).
    pub prev: Option<Arc<BoardState>>,
}

impl BoardState {
    /// Constructs a blank `BoardState`.
    pub const fn blank() -> BoardState {
        BoardState {
            castling: Castling::empty_set(),
            rule_50: 0,
            ply: 0,
            ep_square: NO_SQ,
            psq: Score::ZERO,
            zobrist: 0,
            pawn_key: 0,
            material_key: 0,
            nonpawn_material: [0; PLAYER_CNT],
            captured_piece: PieceType::None,
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
            zobrist: self.zobrist,
            pawn_key: self.pawn_key,
            material_key: self.material_key,
            nonpawn_material: self.nonpawn_material,
            captured_piece: self.captured_piece,
            checkers_bb: BitBoard(0),
            blockers_king: [BitBoard(0); PLAYER_CNT],
            pinners_king: [BitBoard(0); PLAYER_CNT],
            check_sqs: [BitBoard(0); PIECE_TYPE_CNT],
            prev_move: BitMove::null(),
            prev: self.get_prev(),
        }
    }

    /// Sets the current position completely. Used only when initializing a `Board`, not when
    /// applying a move.
    pub(crate) fn set(&mut self, board: &Board) {
        self.zobrist = 0;
        self.material_key = 0;
        self.pawn_key = z_no_pawns();
        self.nonpawn_material = [0; 2];

        let us = board.turn;
        let them = !us;
        let ksq = board.king_sq(us);

        self.checkers_bb =
            board.attackers_to(ksq, board.occupied()) & board.bbs_player[them as usize];

        self.set_check_info(board);
        self.set_zob_hash(board);
        self.set_material_key(board);
    }

    /// Helper method, used after a move is made, creates information concerning checking and
    /// possible checks.
    ///
    /// Specifically, sets Blockers, Pinners, and Check Squares for each piece.
    ///
    /// The `checkers_bb` must beset before this method can be used.
    pub(crate) fn set_check_info(&mut self, board: &Board) {
        let mut white_pinners: BitBoard = BitBoard(0);

        self.blockers_king[Player::White as usize] = board.slider_blockers(
            board.occupied_black(),
            board.king_sq(Player::White),
            &mut white_pinners,
        );

        self.pinners_king[Player::White as usize] = white_pinners;

        let mut black_pinners: BitBoard = BitBoard(0);

        self.blockers_king[Player::Black as usize] = board.slider_blockers(
            board.occupied_white(),
            board.king_sq(Player::Black),
            &mut black_pinners,
        );

        self.pinners_king[Player::Black as usize] = black_pinners;

        let ksq: SQ = board.king_sq(board.turn.other_player());
        let occupied = board.occupied();

        self.check_sqs[PieceType::P as usize] = pawn_attacks_from(ksq, board.turn.other_player());
        self.check_sqs[PieceType::N as usize] = knight_moves(ksq);
        self.check_sqs[PieceType::B as usize] = bishop_moves(occupied, ksq);
        self.check_sqs[PieceType::R as usize] = rook_moves(occupied, ksq);
        self.check_sqs[PieceType::Q as usize] =
            self.check_sqs[PieceType::B as usize] | self.check_sqs[PieceType::R as usize];
        self.check_sqs[PieceType::K as usize] = BitBoard(0);
    }

    // Sets the Zobrist Hash for the current board
    fn set_zob_hash(&mut self, board: &Board) {
        let mut b: BitBoard = board.occupied();
        while let Some(sq) = b.pop_some_lsb() {
            let piece = board.piece_locations.piece_at(sq);
            self.psq += psq(piece, sq);
            let key = z_square(sq, piece);
            self.zobrist ^= key;
            if piece.type_of() == PieceType::P {
                self.pawn_key ^= key;
            }
        }

        self.zobrist ^= z_castle(self.castling.bits());

        let ep = self.ep_square;
        if ep != NO_SQ {
            self.zobrist ^= z_ep(ep);
        }

        match board.turn {
            Player::Black => self.zobrist ^= z_side(),
            Player::White => {}
        };
    }

    /// Sets the material key & Also sets non_pawn material for the board state.
    fn set_material_key(&mut self, board: &Board) {
        for player in &ALL_PLAYERS {
            for piece in &ALL_PIECE_TYPES {
                let count = board.piece_bb(*player, *piece).count_bits();
                for n in 0..count {
                    self.material_key ^= z_square(SQ(n), Piece::make_lossy(*player, *piece));
                }
                if *piece != PieceType::P && *piece != PieceType::K {
                    self.nonpawn_material[*player as usize] +=
                        count as i32 * piecetype_value(*piece, false);
                }
            }
        }
    }

    /// Return the previous BoardState from one move ago.
    ///
    /// If there was no previous state, returns `None`.
    #[inline]
    pub fn get_prev(&self) -> Option<Arc<BoardState>> {
        self.prev.as_ref().cloned()
    }

    /// Iterates through all previous `BoardStates` and prints debug information for each.
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
        print!("ply: {}, move played: {} ", self.ply, self.prev_move);
        if !self.checkers_bb.is_empty() {
            print!("in check {}", self.checkers_bb.to_sq());
        }
        println!();
    }
}

impl PartialEq for BoardState {
    fn eq(&self, other: &BoardState) -> bool {
        self.castling == other.castling
            && self.rule_50 == other.rule_50
            && self.ep_square == other.ep_square
            && self.zobrist == other.zobrist
            && self.captured_piece == other.captured_piece
            && self.checkers_bb == other.checkers_bb
            && self.blockers_king == other.blockers_king
            && self.pinners_king == other.pinners_king
            && self.check_sqs == other.check_sqs
    }
}
