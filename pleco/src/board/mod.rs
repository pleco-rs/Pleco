//! This module contains [`Board`], the object representing the current state of a chessboard.
//! All modifications to the current state of the board is done through this object, as well as
//! gathering information about the current state of the board.
//!
//! This module also contains structures used by the board, such as [`CastlingRights`] for
//! determining castling rights throughout a game. Other utilities that may be of use
//! are [`Eval`], which takes a board and evaluates it, and `PGN` (unstable as of now), which
//! parses a PGN File into a [`Board`].
//!
//! [`Board`]: struct.Board.html
//! [`CastlingRights`]: castle_rights/struct.Castling.html
//! [`Eval`]: eval/struct.Eval.html


use failure;
pub mod movegen;
pub mod castle_rights;
pub mod piece_locations;
pub mod board_state;
pub mod fen;
mod pgn;

extern crate rand;



use core::magic_helper::MagicHelper;
use core::piece_move::{BitMove, MoveType};
use core::move_list::MoveList;
use core::mono_traits::*;
use core::masks::*;
use core::sq::{SQ,NO_SQ};
use core::bitboard::BitBoard;
use core::*;

use tools::prng::PRNG;
use bot_prelude::{IterativeSearcher,JamboreeSearcher};
use tools::Searcher;

use self::castle_rights::Castling;
use self::piece_locations::PieceLocations;
use self::board_state::BoardState;
use self::movegen::{MoveGen,Legal,PseudoLegal};

use std::option::*;
//use std::sync::Arc;
use tools::pleco_arc::{Arc,UniqueArc};
use std::{fmt, char,num};
use std::cmp::{PartialEq,max,min};

pub type Error = failure::Error;




lazy_static! {
    /// Statically initialized lookup tables created when first ran.
    /// Nothing will ever be mutated in here, so it is safe to pass around.
    /// See [`MagicHelper`] for more information.
    ///
    /// [`MagicHelper`]: ../core/magic_helper/struct.MagicHelper.html
    pub static ref MAGIC_HELPER: MagicHelper<'static,'static> = MagicHelper::new();
}

/// Represents possible Errors encountered while building a `Board` from a fen string.
#[derive(Fail, Debug)]
pub enum FenBuildError {
    #[fail(display = "invalid number of fen sections: {}, expected 6", sections)]
    NotEnoughSections {sections: usize},
    #[fail(display = "invalid number of ranks: {}, expected 8", ranks)]
    IncorrectRankAmounts {ranks: usize},
    #[fail(display = "invalid turn: {}, expected 'w' or 'b'", turn)]
    UnrecognizedTurn {turn: String},
    #[fail(display = "unreadable En-passant square: {}", ep)]
    EPSquareUnreadable {ep: String},
    #[fail(display = "invalid En-passant square: {}", ep)]
    EPSquareInvalid {ep: String},
    #[fail(display = "square number too small for rank, rank: {} square: {},", rank, square)]
    SquareSmallerRank {rank: usize, square: String},
    #[fail(display = "square number too large for rank, rank: {} square: {},", rank, square)]
    SquareLargerRank {rank: usize, square: String},
    #[fail(display = "unrecognized piece: {}", piece)]
    UnrecognizedPiece {piece: char},
    #[fail(display = "An unknown error has occurred.")]
    UnreadableMoves(num::ParseIntError),
    #[fail(display = "too many checking piece: {}",num)]
    IllegalNumCheckingPieces{num: u8},
    #[fail(display = "these two pieces cannot check the king at the same time: {}, {}",piece_1, piece_2)]
    IllegalCheckState{piece_1: Piece, piece_2: Piece},
    #[fail(display = "Too many pawns for player: player: {}, # pawns {}",player, num)]
    TooManyPawns{player: Player, num: u8},
    #[fail(display = "Pawn on first or last row")]
    PawnOnLastRow
}

impl From<num::ParseIntError> for FenBuildError {
    fn from(err: num::ParseIntError) -> FenBuildError {
        FenBuildError::UnreadableMoves(err)
    }
}

/// Represents a Chessboard through a `Board`.
///
/// Board contains everything that needs to be known about the current state of the Game. It is used
/// by both Engines and Players / Bots alike.
///
/// Ideally, the Engine contains the original Representation of a board (owns the board), and utilizes
/// `Board::shallow_clone()` to share this representaion with Players.
///
/// # Examples
///
/// ```
/// use pleco::Board;
///
/// fn main() {
///     let mut chessboard = Board::default();
///
///     let moves = chessboard.generate_moves();
///     chessboard.apply_move(moves[0]);
///
///     let b2 = chessboard.shallow_clone(); // boards allow for easy cloning
///     assert_eq!(chessboard.moves_played(), b2.moves_played());
/// }
/// ```
///
/// # `BitBoard` Representation
///
/// For the majority of the struct, the board utilizes [BitBoard]s, which is a u64 where each bit
/// represents an occupied location, and each bit index represents a certain square (as in bit 0 is
/// Square A1, bit 1 is B1, etc.). Indexes increase first horizontally by File, and then by Rank. See
/// [BitBoards article ChessWiki](https://chessprogramming.wikispaces.com/Bitboards) for more information.
///
/// The exact mapping from each square to bits is as follows:
///
/// ```md,ignore
/// 8 | 56 57 58 59 60 61 62 63
/// 7 | 48 49 50 51 52 53 54 55
/// 6 | 40 41 42 43 44 45 46 47
/// 5 | 32 33 34 35 36 37 38 39
/// 4 | 24 25 26 27 28 29 30 31
/// 3 | 16 17 18 19 20 21 22 23
/// 2 | 8  9  10 11 12 13 14 15
/// 1 | 0  1  2  3  4  5  6  7
///   -------------------------
///      a  b  c  d  e  f  g  h
/// ```
pub struct Board {
    turn: Player, // Current turn
    bit_boards: [[BitBoard; PIECE_CNT]; PLAYER_CNT], // Occupancy per player per piece
    occ: [BitBoard; PLAYER_CNT], // Occupancy per Player
    occ_all: BitBoard, // BitBoard of all pieces
    half_moves: u16, // Total moves played
    depth: u16, // Current depth since last shallow_copy
    piece_counts: [[u8; PIECE_CNT]; PLAYER_CNT], // Count of each Piece
    piece_locations: PieceLocations, // Mapping Squares to Pieces and Plauers

    // State of the Board, Un modifiable.
    // Arc to allow easy and quick copying of boards without copying memory
    // or recomputing BoardStates.
    state: Arc<BoardState>,

    /// Reference to the pre-computed lookup tables.
    #[doc(hidden)]
    pub magic_helper: &'static MagicHelper<'static, 'static>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}


impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Board: {}", &self.pretty_string())
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        self.turn == other.turn &&
            self.occ_all == other.occ_all &&
            *self.state == *other.state &&
            self.piece_locations == other.piece_locations
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        self.shallow_clone()
    }
}

impl Board {
    /// Constructs a board from the starting position
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.count_pieces_player(Player::White),16);
    /// ```
    pub fn default() -> Board {
        let mut b = Board {
            turn: Player::White,
            bit_boards: BitBoard::start_bbs(),
            occ: [BitBoard(START_WHITE_OCC), BitBoard(START_BLACK_OCC)],
            occ_all: BitBoard(START_OCC_ALL),
            half_moves: 0,
            depth: 0,
            piece_counts: [[8, 2, 2, 2, 1, 1], [8, 2, 2, 2, 1, 1]],
            piece_locations: PieceLocations::default(),
            state: Arc::new(BoardState::default()),
            magic_helper: &MAGIC_HELPER,
        };
        // Create the Zobrist hash & set the Piece Locations structure
        b.set_zob_hash();
        b.set_piece_states();
        b
    }

    /// Constructs a shallow clone of the Board.
    ///
    /// Contains only the information necessary to apply future moves, more specifically
    /// does not clone the moves list, and sets depth to zero. Intended for an Engine or
    /// main thread to share the board to users wanting to search.
    ///
    /// # Safety
    ///
    /// After this method has called, [Board::undo_move()] cannot be called immediately after.
    /// Undoing moves can only be done once a move has been played, and cannot be called more
    /// times than moves have been played since calling [Board::shallow_clone()].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let mut chessboard = Board::default();
    /// let moves = chessboard.generate_moves(); // generate all possible legal moves
    /// chessboard.apply_move(moves[0]); // apply first move
    ///
    /// assert_eq!(chessboard.moves_played(), 1);
    ///
    /// let board_clone = chessboard.shallow_clone();
    /// assert_eq!(chessboard.moves_played(), board_clone.moves_played());
    ///
    /// assert_ne!(chessboard.depth(),board_clone.depth()); // different depths
    /// ```
    pub fn shallow_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: BitBoard::clone_all_occ(&self.bit_boards),
            occ:  BitBoard::clone_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: 0,
            piece_counts: self.piece_counts,
            piece_locations: self.piece_locations.clone(),
            state: Arc::clone(&self.state),
            magic_helper: &MAGIC_HELPER,
        }
    }

    /// Constructs a parallel clone of the Board.
    ///
    /// Similar to [Board::shallow_clone()], but keeps the current search depth the same.
    /// Should be used when implementing a searcher, and want to search a list of moves
    /// in parallel with different threads.
    ///
    /// # Safety
    ///
    /// After this method has called, [Board::undo_move()] cannot be called immediately after.
    /// Undoing moves can only be done once a move has been played, and cannot be called more
    /// times than moves have been played since calling [Board::parallel_clone()].
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let mut chessboard = Board::default();
    /// let moves = chessboard.generate_moves(); // generate all possible legal moves
    /// chessboard.apply_move(moves[0]);
    /// assert_eq!(chessboard.moves_played(), 1);
    ///
    /// let board_clone = chessboard.parallel_clone();
    /// assert_eq!(chessboard.moves_played(), board_clone.moves_played());
    ///
    /// assert_eq!(chessboard.depth(),board_clone.depth()); // different depths
    /// ```
    pub fn parallel_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: BitBoard::clone_all_occ(&self.bit_boards),
            occ: BitBoard::clone_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: self.depth,
            piece_counts: self.piece_counts.clone(),
            piece_locations: self.piece_locations.clone(),
            state: Arc::clone(&self.state),
            magic_helper: &MAGIC_HELPER,
        }
    }

    /// Creates a `RandBoard` (Random Board Generator) for generation of `Board`s with random
    /// positions. See the `RandBoard` structure for more information.
    ///
    /// # Examples
    ///
    /// Create one `Board` with at least 5 moves played that is created in a pseudo-random
    /// fashion.
    ///
    /// ```
    /// use pleco::Board;
    /// let rand_boards: Board = Board::random()
    ///     .pseudo_random(12455)
    ///     .min_moves(5)
    ///     .one();
    /// ```
    ///
    /// Create a `Vec` of 10 random `Board`s that are guaranteed to not be in check.
    ///
    /// ```
    /// use pleco::board::{Board,RandBoard};
    ///
    /// let rand_boards: Vec<Board> = Board::random()
    ///     .pseudo_random(12455)
    ///     .no_check()
    ///     .many(10);
    /// ```
    pub fn random() -> RandBoard {
        RandBoard::default()
    }

    /// Helper method for setting the piece states on initialization.
    ///
    /// Only used when creating the Board from scratch (e.g. default position).
    ///
    /// # Safety
    ///
    /// Assumes that the Board has all of its BitBoards completely set, including the BitBoards
    /// for the individual pieces as well as occupancy per player BitBoards.
    fn set_piece_states(&mut self) {
        // Loop each piece and player and count all the pieces per player
        for player in &ALL_PLAYERS {
            for piece in &ALL_PIECES {
                self.piece_counts[*player as usize][*piece as usize] =
                    self.piece_bb(*player, *piece).count_bits();
            }
        }

        // Loop through each square and see if any bitboard contains something at that location, and set
        // the Boards' PieceLocations accordingly.
        for square in 0..SQ_CNT as u8 {
            let bb = SQ(square).to_bb();
            if (bb & self.get_occupied()).is_not_empty() {
                let player = if (bb & self.occupied_black()).is_empty() {
                    Player::White
                } else {
                    Player::Black
                };
                let piece = if (self.piece_bb(player, Piece::P) & bb).is_not_empty() {
                    Piece::P
                } else if (self.piece_bb(player, Piece::N) & bb).is_not_empty() {
                    Piece::N
                } else if (self.piece_bb(player, Piece::B) & bb).is_not_empty() {
                    Piece::B
                } else if (self.piece_bb(player, Piece::R) & bb).is_not_empty() {
                    Piece::R
                } else if (self.piece_bb(player, Piece::Q) & bb).is_not_empty() {
                    Piece::Q
                } else if (self.piece_bb(player, Piece::K) & bb).is_not_empty() {
                    Piece::K
                } else {
                    panic!()
                };
                self.piece_locations.place(SQ(square), player, piece);
            } else {
                // Remove the square just in case nothing eas found. Can't assume that the PieceLocations
                // represents that square as blank
                self.piece_locations.remove(SQ(square));
            }
        }
    }


    /// Helper method for setting the BitBoards from a fully created PieceLocations.
    ///
    /// Only used when creating the Board from a fen String.
    ///
    /// # Safety
    ///
    /// Assumes that the Board has its PieceLocations completely set.
    fn set_bitboards(&mut self) {
        for sq in 0..SQ_CNT as u8 {
            if let Some((player, piece)) = self.piece_locations.player_piece_at(SQ(sq)) {
                let bb = SQ(sq).to_bb();
                self.bit_boards[player as usize][piece as usize] |= bb;
                self.occ[player as usize] |= bb;
            }
        }
        self.occ_all = self.occupied_black() | self.occupied_white();
        for player in &ALL_PLAYERS {
            for piece in &ALL_PIECES {
                self.piece_counts[*player as usize][*piece as usize] =
                    self.piece_bb(*player, *piece).count_bits();
            }
        }
    }


    /// Constructs a board from a FEN String.
    ///
    /// FEN stands for Forsyth-Edwards Notation, and is a way of representing a board through a
    /// string of characters. More information can be found on the [ChessWiki](https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    /// assert_eq!(board.count_all_pieces(),32);
    /// ```
    ///
    /// # Panics
    ///
    /// The FEN string must be valid, or else the method will panic.
    ///
    /// There is a possibility of the FEN string representing an unvalid position, with no panics resulting.
    /// The Constructed Board may have some Undefined Behavior as a result. It is up to the user to give a
    /// valid FEN string.
    pub fn new_from_fen(fen: &str) -> Result<Board,FenBuildError> {

        // split the string by white space
        let det_split: Vec<&str> = fen.split_whitespace().collect();

        // must have 6 parts :
        // [ Piece Placement, Side to Move, Castling Ability, En Passant square, Half moves, full moves]
        if det_split.len() != 6 {
            return Err(FenBuildError::NotEnoughSections{sections: det_split.len()});
        }

        // Split the first part by '/' for locations
        let b_rep: Vec<&str> = det_split[0].split('/').collect();

        if b_rep.len() != 8 {
            return Err(FenBuildError::IncorrectRankAmounts{ranks: b_rep.len()});
        }

        let (piece_loc, piece_cnt) = PieceLocations::from_partial_fen(b_rep.as_slice())?;

        // Side to Move
        let turn_char: char = det_split[1].chars()
            .next()
            .ok_or(FenBuildError::UnrecognizedTurn{turn: det_split[1].to_string()})?;
        let turn: Player = match turn_char {
            'b' => Player::Black,
            'w' => Player::White,
            _ => {return Err(FenBuildError::UnrecognizedTurn{turn: det_split[1].to_string()});},
        };

        // Castle Bytes
        let mut castle_bytes = Castling::empty();
        for char in det_split[2].chars() {
            castle_bytes.add_castling_char(char);
        }

        let mut ep_sq: SQ = SQ(0);
        for (i, character) in det_split[3].chars().enumerate() {
            if i > 1 { return Err(FenBuildError::EPSquareUnreadable{ep: det_split[3].to_string()}); }
            if i == 0 {
                match character {
                    'a' => ep_sq += SQ(0),
                    'b' => ep_sq += SQ(1),
                    'c' => ep_sq += SQ(2),
                    'd' => ep_sq += SQ(3),
                    'e' => ep_sq += SQ(4),
                    'f' => ep_sq += SQ(5),
                    'g' => ep_sq += SQ(6),
                    'h' => ep_sq += SQ(7),
                    '-' => {}
                    _ => { return Err(FenBuildError::EPSquareUnreadable{ep: det_split[3].to_string()}); }
                }
            } else {
                let digit = character.to_digit(10)
                    .ok_or( FenBuildError::EPSquareUnreadable{ep: det_split[3].to_string()})? as u8;
                // must be 3 or 6

                if digit == 3 {
                    ep_sq += SQ(16);  // add two ranks
                } else if digit == 6 {
                    ep_sq += SQ(40);
                } else {
                    return Err(FenBuildError::EPSquareInvalid{ep: det_split[3].to_string()});
                }
            }
        }

        if ep_sq == SQ(0) {ep_sq = NO_SQ}

        // rule 50 counts
        let rule_50 = det_split[4].parse::<i16>()?;

        // Total Moves Played
        // Moves is defined as everyime White moves, so gotta translate to total moves
        let mut total_moves = (det_split[5].parse::<u16>()? - 1) * 2;
        if turn == Player::Black {
            total_moves += 1
        };

        // Create the Board States
        let mut board_s = UniqueArc::new(BoardState {
            castling: castle_bytes,
            rule_50: rule_50,
            ply: 0,
            ep_square: ep_sq,
            zobrast: 0,
            pawn_key: 0,
            captured_piece: None,
            checkers_bb: BitBoard(0),
            blockers_king: [BitBoard(0); PLAYER_CNT],
            pinners_king: [BitBoard(0); PLAYER_CNT],
            check_sqs: [BitBoard(0); PIECE_CNT],
            prev_move: BitMove::null(),
            prev: None,
        });

        // Create the Board
        let mut b = Board {
            turn: turn,
            bit_boards: [[BitBoard(0); PIECE_CNT]; PLAYER_CNT],
            occ: [BitBoard(0), BitBoard(0)],
            occ_all: BitBoard(0),
            half_moves: total_moves,
            depth: 0,
            piece_counts: piece_cnt,
            piece_locations: piece_loc,
            state: Arc::new(BoardState::default()),
            magic_helper: &MAGIC_HELPER,
        };

        // Set the BitBoards
        b.set_bitboards();
        { // Set Check info
            let state: &mut BoardState = &mut *board_s;
            b.set_check_info(state);
        }
        b.state = board_s.shareable();
        b.set_zob_hash();

        fen::is_valid_fen(b)
    }

    /// Creates a FEN String of the Given Board.
    ///
    /// FEN stands for Forsyth-Edwards Notation, and is a way of representing a board through a
    /// string of characters. More information can be found on the [ChessWiki](https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let board = Board::default();
    /// assert_eq!(board.get_fen(),"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// ```
    pub fn get_fen(&self) -> String {
        let mut s = String::default();
        let mut blanks = 0;
        for idx in 0..SQ_CNT as u8 {
            // Cause of weird fen ordering, gotta do it this way
            let sq = SQ((idx % 8) + (8 * (7 - (idx / 8))));
            if sq.file() == File::A && sq.rank() != Rank::R8 {
                if blanks != 0 {
                    // Only add a number if there is a space between pieces
                    s.push(char::from_digit(blanks, 10).unwrap());
                    blanks = 0;
                }
                s.push('/');
            }
            let piece = self.piece_at_sq(sq);
            let player = self.player_at_sq(sq);
            if piece.is_none() {
                blanks += 1;
            } else {
                if blanks != 0 {
                    s.push(char::from_digit(blanks, 10).unwrap());
                    blanks = 0;
                }
                s.push(
                    PIECE_DISPLAYS[player.unwrap() as usize][piece.unwrap() as usize],
                );
            }
        }

        if blanks != 0 {
            s.push(char::from_digit(blanks, 10).unwrap());
        }

        s.push(' ');
        // current turn
        s.push(match self.turn {
            Player::White => 'w',
            Player::Black => 'b',
        });
        s.push(' ');

        // Castling State
        s.push_str(&(self.state.castling.pretty_string()));
        s.push(' ');

        // EP Square
        if self.ep_square() == NO_SQ {
            s.push('-');
        } else {
            let ep = self.ep_square();
            s.push(FILE_DISPLAYS[ep.file_idx_of_sq() as usize]);
            s.push(RANK_DISPLAYS[ep.rank_idx_of_sq() as usize]);
        }
        s.push(' ');
        s.push_str(&format!("{}", self.rule_50()));
        s.push(' ');
        s.push_str(&format!("{}", (self.half_moves / 2) + 1));

        s
    }

    /// Applies a move to the Board.
    ///
    /// # Safety
    ///
    /// The passed in [BitMove] must be a legal move for the current position.
    ///
    /// # Panics
    ///
    /// The supplied BitMove must be both a valid move for that position, as well as a
    /// valid [BitMove], Otherwise, a panic will occur. Valid BitMoves can be generated with
    /// [Board::generate_moves()], which guarantees that only Legal moves will be created.
    pub fn apply_move(&mut self, bit_move: BitMove) {
        let gives_check: bool = self.gives_check(bit_move);
        self.apply_unknown_move(bit_move, gives_check);
    }

    /// Applies a move to the Board. This method is only useful if before a move is applied to
    /// a board, the ability of the move to give check is applied. If it is not needed to know
    /// if the move gives check or not, consider using `Board::apply_move` instead.
    ///
    /// # Safety
    ///
    /// The passed in [BitMove] must be a legal move for the current position.
    ///
    /// # Panics
    ///
    /// The supplied BitMove must be both a valid move for that position, as well as a
    /// valid [BitMove], Otherwise, a panic will occur. Valid BitMoves can be generated with
    /// [Board::generate_moves()], which guarantees that only Legal moves will be created.
    ///
    /// The second parameter, `gives_check`, must be true if the move gives check, or false
    /// if the move doesn't give check. If an incorrect `gives_check` is supplied, undefined
    /// behavior will follow.
    pub fn apply_unknown_move(&mut self, bit_move: BitMove, gives_check: bool) {

        // TODO: investigate potention for SIMD in capturing moves
        //
        // Specifically https://github.com/rust-lang-nursery/simd, u16 X 8 ?

        // Check for stupidity
        assert_ne!(bit_move.get_src(), bit_move.get_dest());

        // Zobrist Hash
        let mut pawn_key: u64 = self.state.pawn_key;
        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;


        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = UniqueArc::new(self.state.partial_clone());

        {
            // Seperate Block to allow derefencing the BoardState
            // As there is garunteed only one owner of the Arc, this is allowed
            let new_state: &mut BoardState = &mut *next_arc_state;

            // Set the prev state
            new_state.prev = Some(Arc::clone(&self.state));

            // Increment these
            self.half_moves += 1;
            self.depth += 1;
            new_state.rule_50 += 1;
            new_state.ply += 1;
            new_state.prev_move = bit_move;


            let us = self.turn;
            let them = us.other_player();
            let from: SQ = bit_move.get_src();
            let mut to: SQ = bit_move.get_dest();
            let piece: Piece = self.piece_at_sq(from).unwrap();

            let captured: Option<Piece> = if bit_move.is_en_passant() {
                Some(Piece::P)
            } else {
                self.piece_at_sq(to)
            };

            // Sanity checks
            assert_eq!(self.color_of_sq(from).unwrap(), us);

            if bit_move.is_castle() {

                // Sanity Checks, moved piece should be K, "captured" should be R
                // As this is the encoding of Castling
                assert_eq!(captured.unwrap(), Piece::R);
                assert_eq!(piece, Piece::K);

                let mut r_src: SQ = SQ(0);
                let mut r_dst: SQ = SQ(0);

                // yay helper methods
                self.apply_castling(us, from, &mut to, &mut r_src, &mut r_dst);

                zob ^= self.magic_helper.z_piece_at_sq(Piece::R, r_src) ^
                    self.magic_helper.z_piece_at_sq(Piece::R, r_dst);
                new_state.captured_piece = None;
                new_state.castling.set_castling(us);
            } else if let Some(cap_p) = captured {
                let mut cap_sq: SQ = to;
                if cap_p == Piece::P {
                    pawn_key ^= self.magic_helper.z_piece_at_sq(cap_p, cap_sq);
                    if bit_move.is_en_passant() {
                        assert_eq!(cap_sq, self.state.ep_square);
                        match us {
                            Player::White => cap_sq -= SQ(8),
                            Player::Black => cap_sq += SQ(8),
                        };
                        assert_eq!(piece, Piece::P);
                        assert_eq!(us.relative_rank( Rank::R6), to.rank());
                        assert!(self.piece_at_sq(to).is_none());
                        assert_eq!(self.piece_at_sq(cap_sq).unwrap(), Piece::P);
                        assert_eq!(self.player_at_sq(cap_sq).unwrap(), them);
                        self.remove_piece_c(Piece::P, cap_sq, them);
                    } else {
                        self.remove_piece_c(cap_p, cap_sq, them);
                    }
                } else {
                    self.remove_piece_c(cap_p, cap_sq, them);
                }
                zob ^= self.magic_helper.z_piece_at_sq(cap_p, cap_sq);

                // Reset Rule 50
                new_state.rule_50 = 0;
                new_state.captured_piece = Some(cap_p);
            }

            // Update hash for moving piece
            zob ^= self.magic_helper.z_piece_at_sq(piece, to) ^
                self.magic_helper.z_piece_at_sq(piece, from);

            if self.state.ep_square != NO_SQ {
                zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            // Update castling rights
            if !new_state.castling.is_empty() && (to.castle_rights_mask() | from.castle_rights_mask()) != 0 {
                let castle_zob_index = new_state.castling.update_castling(to,from);
                zob ^= self.magic_helper.z_castle_rights(castle_zob_index);
            }

            // Actually move the piece
            if !bit_move.is_castle()  {
                self.move_piece_c(piece, from, to, us);
            }

            // Pawn Moves need special help :(
            if piece == Piece::P {
                if self.magic_helper.distance_of_sqs(to, from) == 2 {
                    // Double Push
                    new_state.ep_square = (to + from) / SQ(2);
                    zob ^= self.magic_helper.z_ep_file(new_state.ep_square);
                } else if bit_move.is_promo() {
                    let promo_piece: Piece = bit_move.promo_piece();

                    self.remove_piece_c(piece, to, us);
                    self.put_piece_c(promo_piece, to, us);
                    zob ^= self.magic_helper.z_piece_at_sq(promo_piece, to) ^
                        self.magic_helper.z_piece_at_sq(Piece::P, from);
                    pawn_key ^= self.magic_helper.z_piece_at_sq(piece, to);
                }
                pawn_key ^= self.magic_helper.z_piece_at_sq(piece, from) ^ self.magic_helper.z_piece_at_sq(piece, to);
                new_state.rule_50 = 0;
            }

            new_state.captured_piece = captured;
            new_state.zobrast = zob;
            new_state.pawn_key = pawn_key;

            new_state.checkers_bb = if gives_check {
                self.attackers_to(self.king_sq(them), self.get_occupied()) &
                    self.get_occupied_player(us)
            } else {
                BitBoard(0)
            };

            self.turn = them;
            self.set_check_info(new_state); // Set the checking information
        }
        self.state = next_arc_state.shareable();

        if cfg!(debug_assertions) {
            self.is_okay().unwrap();
        } else {
            assert!(self.is_ok_quick());
        }
    }

    /// Applies a UCI move to the board. If the move is a valid string representing a UCI move, then
    /// true will be returned & the move will be applied. Otherwise, false is returned and the board isn't
    /// changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let mut board = Board::default();
    /// let success = board.apply_uci_move("e2e4");
    ///
    /// assert!(success);
    /// ```
    pub fn apply_uci_move(&mut self, uci_move: &str) -> bool {
        let all_moves: MoveList = self.generate_moves();
        let bit_move: Option<BitMove> = all_moves.iter()
                                                 .find(|m| m.stringify() == uci_move)
                                                 .cloned();
        if let Some(mov) = bit_move {
            self.apply_move(mov);
            return true;
        }
        false
    }

    /// Un-does the previously applied move, allowing the Board to return to it's most recently held state.
    ///
    /// # Panics
    ///
    /// Cannot be done if after any `Board::shallow_clone()` has been applied,
    /// or if `Board::parallel_clone()` has been done and there is no previous move.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use pleco::Board;
    ///
    /// let mut chessboard = Board::default();
    ///
    /// let moves = chessboard.generate_moves();
    /// chessboard.apply_move(moves[0]);
    ///
    /// let mut board_clone = chessboard.shallow_clone();
    ///
    /// chessboard.undo_move(); // works, chessboard existed before the move was played
    /// board_clone.undo_move(); // error: board_clone was created after the move was applied
    ///
    /// ```
    pub fn undo_move(&mut self) {
        assert!(self.state.prev.is_some());
        assert!(!self.state.prev_move.is_null());

        let undo_move: BitMove = self.state.prev_move;

        self.turn = self.turn.other_player();
        let us: Player = self.turn;
        let from: SQ = undo_move.get_src();
        let to: SQ = undo_move.get_dest();
        let mut piece_on: Option<Piece> = self.piece_at_sq(to);

        // Make sure the piece moved from is not there, or there is a castle
        assert!(self.piece_at_sq(from).is_none() || undo_move.is_castle());

        if undo_move.is_promo() {
            assert_eq!(piece_on.unwrap(), undo_move.promo_piece());

            self.remove_piece_c(piece_on.unwrap(), to, us);
            self.put_piece_c(Piece::P, to, us);
            piece_on = Some(Piece::P);

        }

        if undo_move.is_castle() {
            self.remove_castling(us, from, to);
        } else {
            self.move_piece_c(piece_on.unwrap(), to, from, us);
            let cap_piece = self.state.captured_piece;

            if cap_piece.is_some() {
                let mut cap_sq: SQ = to;
                if undo_move.is_en_passant() {
                    match us {
                        Player::White => cap_sq -= SQ(8),
                        Player::Black => cap_sq += SQ(8),
                    };
                }
                self.put_piece_c(cap_piece.unwrap(), cap_sq, us.other_player());
            }
        }
        self.state = self.state.get_prev().unwrap();
        self.half_moves -= 1;
        self.depth -= 1;

        if cfg!(debug_assertions) {
            self.is_okay().unwrap();
        } else {
            assert!(self.is_ok_quick());
        }
    }

    /// Apply a "Null Move" to the board, essentially swapping the current turn of
    /// the board without moving any pieces.
    ///
    /// # Safety
    ///
    /// This method should only be used for special evaluation purposes, as it does not give an
    /// accurate or legal state of the chess board.
    ///
    /// Unsafe as it allows for Null Moves to be applied in states of check, which is never a valid
    /// state of a chess game.
    ///
    /// # Panics
    ///
    /// Panics if the Board is currently in check.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::board::*;
    ///
    /// let mut chessboard = Board::default();
    /// let board_clone = chessboard.shallow_clone();
    ///
    /// unsafe { chessboard.apply_null_move(); }
    ///
    /// assert_ne!(chessboard.depth(), board_clone.depth());
    /// ```
    pub unsafe fn apply_null_move(&mut self) {
        assert!(self.checkers().is_empty());

        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;

        self.depth += 1;
        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = UniqueArc::new(self.state.partial_clone());

        {
            let new_state: &mut BoardState = &mut *next_arc_state;

            new_state.prev_move = BitMove::null();
            new_state.rule_50 += 1;
            new_state.ply += 1;

            new_state.prev = Some(Arc::clone(&self.state));

            if self.state.ep_square != NO_SQ {
                zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            new_state.zobrast = zob;
            self.turn = self.turn.other_player();
            self.set_check_info(new_state);
        }
        self.state = next_arc_state.shareable();

        if cfg!(debug_assertions) {
            self.is_okay().unwrap();
        } else {
            assert!(self.is_ok_quick());
        }
    }

    /// Undo a "Null Move" to the Board, returning to the previous state.
    ///
    /// # Safety
    ///
    /// This method should only be used if it can be guaranteed that the last played move from
    /// the current state is a Null-Move, eg `Board::apply_null_move()`. Otherwise, a panic will occur.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::board::*;
    ///
    /// let mut chessboard = Board::default();
    /// let board_clone = chessboard.shallow_clone();
    ///
    /// unsafe { chessboard.apply_null_move(); }
    ///
    /// assert_ne!(chessboard.ply(), board_clone.ply());
    ///
    /// unsafe { chessboard.undo_null_move(); }
    ///
    /// assert_eq!(chessboard.moves_played(), board_clone.moves_played());
    /// assert_eq!(chessboard.get_fen(), board_clone.get_fen());
    /// ```
    pub unsafe fn undo_null_move(&mut self) {
        assert!(self.state.prev_move.is_null());
        self.turn = self.turn.other_player();
        self.state = self.state.get_prev().unwrap();
    }

    /// Get a List of legal `BitMove`s for the player whose turn it is to move.
    ///
    /// This method already takes into account if the Board is currently in check, and will return
    /// legal moves only.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let chessboard = Board::default();
    /// let moves = chessboard.generate_moves();
    ///
    /// println!("There are {} possible legal moves.", moves.len());
    /// ```
    pub fn generate_moves(&self) -> MoveList {
        MoveGen::generate::<Legal, AllGenType>(self)
    }

    /// Get a List of all PseudoLegal `BitMove`s for the player whose turn it is to move.
    /// Works exactly the same as `Board::generate_moves()`, but doesn't guarantee that all
    /// the moves are legal for the current position. Moves need to be checked with a
    /// `Board::legal_move(move)` in order to be certain of a legal move.
    pub fn generate_pseudolegal_moves(&self) -> MoveList {
        MoveGen::generate::<PseudoLegal, AllGenType>(self)
    }

    /// Get a List of legal `BitMove`s for the player whose turn it is to move and of a certain type.
    ///
    /// This method already takes into account if the Board is currently in check, and will return
    /// legal moves only. If a non-ALL `GenTypes` is supplied, only a subset of the total moves will be given.
    ///
    /// # Panics
    ///
    /// Panics if given `GenTypes::QuietChecks` while the current board is in check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pleco::board::*;
    /// use pleco::core::GenTypes;
    ///
    /// let chessboard = Board::default();
    /// let capturing_moves = chessboard.generate_moves_of_type(GenTypes::Captures);
    ///
    /// assert_eq!(capturing_moves.len(), 0); // no possible captures for the starting position
    /// ```
    pub fn generate_moves_of_type(&self, gen_type: GenTypes) -> MoveList {
        match gen_type {
            GenTypes::All => MoveGen::generate::<Legal,AllGenType>(self),
            GenTypes::Captures => MoveGen::generate::<Legal,CapturesGenType>(self),
            GenTypes::Quiets => MoveGen::generate::<Legal,QuietsGenType>(self),
            GenTypes::QuietChecks => MoveGen::generate::<Legal,QuietChecksGenType>(self),
            GenTypes::Evasions => MoveGen::generate::<Legal,EvasionsGenType>(self),
            GenTypes::NonEvasions => MoveGen::generate::<Legal,NonEvasionsGenType>(self)
        }
    }

    /// Get a List of all PseudoLegal `BitMove`s for the player whose turn it is to move.
    /// Works exactly the same as `Board::generate_moves()`, but doesn't guarantee that all
    /// the moves are legal for the current position. Moves need to be checked with a
    /// `Board::legal_move(move)` in order to be certain of a legal move.
    ///
    /// This method already takes into account if the Board is currently in check.
    /// If a non-ALL GenType is supplied, only a subset of the total moves will be given.
    ///
    /// # Panics
    ///
    /// Panics if given `GenTypes::QuietChecks` while the current board is in check
    pub fn generate_pseudolegal_moves_of_type(&self, gen_type: GenTypes) -> MoveList {
        match gen_type {
            GenTypes::All => MoveGen::generate::<PseudoLegal,AllGenType>(self),
            GenTypes::Captures => MoveGen::generate::<PseudoLegal,CapturesGenType>(self),
            GenTypes::Quiets => MoveGen::generate::<PseudoLegal,QuietsGenType>(self),
            GenTypes::QuietChecks => MoveGen::generate::<PseudoLegal,QuietChecksGenType>(self),
            GenTypes::Evasions => MoveGen::generate::<PseudoLegal,EvasionsGenType>(self),
            GenTypes::NonEvasions => MoveGen::generate::<PseudoLegal,NonEvasionsGenType>(self)
        }
    }

    //  ------- PRIVATE MUTATING FUNCTIONS -------

    /// Helper method, used after a move is made, creates information concerning checking and
    /// possible checks.
    ///
    /// Specifically, sets Blockers, Pinners, and Check Squares for each piece.
    fn set_check_info(&self, board_state: &mut BoardState) {

        // Set the Pinners and Blockers
        let mut white_pinners: BitBoard = BitBoard(0);
        // TODO: NLL
        {
            board_state.blockers_king[Player::White as usize] = self.slider_blockers(
                self.occupied_black(),
                self.king_sq(Player::White),
                &mut white_pinners,
            )
        };

        board_state.pinners_king[Player::White as usize] = white_pinners;

        // TODO: NLL
        let mut black_pinners: BitBoard = BitBoard(0);
        {
            board_state.blockers_king[Player::Black as usize] = self.slider_blockers(
                self.occupied_white(),
                self.king_sq(Player::Black),
                &mut black_pinners,
            )
        };

        board_state.pinners_king[Player::Black as usize] = black_pinners;

        let ksq: SQ = self.king_sq(self.turn.other_player());
        let occupied = self.get_occupied();

        board_state.check_sqs[Piece::P as usize] = self.magic_helper
                                                       .pawn_attacks_from(ksq, self.turn.other_player());
        board_state.check_sqs[Piece::N as usize] = self.magic_helper.knight_moves(ksq);
        board_state.check_sqs[Piece::B as usize] = self.magic_helper.bishop_moves(occupied, ksq);
        board_state.check_sqs[Piece::R as usize] = self.magic_helper.rook_moves(occupied, ksq);
        board_state.check_sqs[Piece::Q as usize] = board_state.check_sqs[Piece::B as usize] |
            board_state.check_sqs[Piece::R as usize];
        board_state.check_sqs[Piece::K as usize] = BitBoard(0);
    }



    /// Removes a Piece from the Board, if the color is unknown.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn remove_piece(&mut self, piece: Piece, square: SQ) {
        let player = self.color_of_sq(square).unwrap();
        self.remove_piece_c(piece, square, player);
    }

    /// Moves a Piece on the Board (if the color is unknown) from square 'from'
    /// to square 'to'.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn move_piece(&mut self, piece: Piece, from: SQ, to: SQ) {
        let player = self.color_of_sq(from).unwrap();
        self.move_piece_c(piece, from, to, player);
    }

    /// Places a Piece on the board at a given square and player.
    ///
    /// # Safety
    ///
    /// Assumes there is not already a piece at that square. If there already is,
    /// Undefined Behavior will result.
    fn put_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        let bb = square.to_bb();
        self.occ_all |= bb;
        self.occ[player as usize] |= bb;
        self.bit_boards[player as usize][piece as usize] |= bb;

        self.piece_locations.place(square, player, piece);
        self.piece_counts[player as usize][piece as usize] += 1;
        // Note: Should We set captured Piece?
    }

    /// Removes a Piece from the Board for a given player.
    ///
    /// # Panics
    ///
    /// Panics if there is a piece at the given square.
    fn remove_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        assert_eq!(self.piece_at_sq(square).unwrap(), piece);
        let bb = square.to_bb();
        self.occ_all ^= bb;
        self.occ[player as usize] ^= bb;
        self.bit_boards[player as usize][piece as usize] ^= bb;

        self.piece_locations.remove(square);
        self.piece_counts[player as usize][piece as usize] -= 1;
    }

    /// Moves a Piece on the Board of a given player from square 'from'
    /// to square 'to'.
    ///
    /// # Panics
    ///
    /// Panics if the two and from square are equal
    fn move_piece_c(&mut self, piece: Piece, from: SQ, to: SQ, player: Player) {
        assert_ne!(from, to);
        let comb_bb: BitBoard = from.to_bb() | to.to_bb();

        self.occ_all ^= comb_bb;
        self.occ[player as usize] ^= comb_bb;
        self.bit_boards[player as usize][piece as usize] ^= comb_bb;

        self.piece_locations.remove(from);
        self.piece_locations.place(to, player, piece);
    }

    /// Helper function to apply a Castling for a given player.
    ///
    /// Takes in the player to castle, alongside the original king square and the original rook square.
    /// the k_dst and r_dst squares are pointers to values, modifying them to have the correct king and
    /// rook destination squares.
    ///
    /// # Safety
    ///
    /// Assumes that k_src and r_src are legal squares, and the player can legally castle.
    fn apply_castling(
        &mut self,
        player: Player,
        k_src: SQ,    // from, king startng spot
        to_r_orig: &mut SQ, // originally
        r_src: &mut SQ,
        r_dst: &mut SQ,
    ) {
        let king_side: bool = k_src < *to_r_orig;

        *r_src = *to_r_orig;
        if king_side {
            *to_r_orig = player.relative_square( SQ(6));
            *r_dst = player.relative_square( SQ(5));
        } else {
            *to_r_orig = player.relative_square( SQ(2));
            *r_dst = player.relative_square( SQ(3));
        }
        self.move_piece_c(Piece::K, k_src, *to_r_orig, player);
        self.move_piece_c(Piece::R, *r_src, *r_dst, player);
    }

    /// Helper function to remove a Castling for a given player.
    ///
    /// Takes in the player to castle, alongside the post-castle king rook squares.
    ///
    /// # Safety
    ///
    /// Assumes the last move played was a castle for the given player.
    fn remove_castling(&mut self, player: Player, k_src: SQ, r_src: SQ) {
        let k_dst: SQ = self.king_sq(player);
        let king_side: bool = k_src < r_src;
        let r_dst: SQ = if king_side {
            player.relative_square(SQ(5))
        } else {
            player.relative_square(SQ(3))
        };

        self.move_piece_c(Piece::K, k_dst, k_src, player);
        self.move_piece_c(Piece::R, r_dst, r_src, player);
    }

    /// Helper function to that outputs the Blockers of a given square
    fn slider_blockers(&self, sliders: BitBoard, s: SQ, pinners: &mut BitBoard) -> BitBoard {
        let mut result: BitBoard = BitBoard(0);
        *pinners = BitBoard(0);
        let occupied: BitBoard = self.get_occupied();

        let mut snipers: BitBoard = sliders &
            ((self.magic_helper.rook_moves(BitBoard(0), s) &
                (self.piece_two_bb_both_players(Piece::R, Piece::Q))) |
                (self.magic_helper.bishop_moves(BitBoard(0), s) &
                    (self.piece_two_bb_both_players(Piece::B, Piece::Q))));


        while snipers.is_not_empty() {
            let sniper_sq: SQ = snipers.pop_lsb();
            let b: BitBoard = self.magic_helper.between_bb(s, sniper_sq) & occupied;
            if !b.more_than_one() {
                result |= b;
                let other_occ = self.get_occupied_player(self.player_at_sq(s).unwrap());
                if (b & other_occ).is_not_empty() {
                    *pinners |= sniper_sq.to_bb();
                }
            }
        }

        result
    }

    //    pub struct Zobrist {
    //      sq_piece: [[u64; PIECE_CNT]; SQ_CNT],
    //      en_p: [u64; FILE_CNT],
    //      castle: [u64; CASTLING_CNT],
    //      side: u64,
    //    }

    /// Sets the Zobrist hash when the board is initialized or created from a FEN string.
    ///
    /// Assumes the rest of the board is initialized.
    fn set_zob_hash(&mut self) {
        let mut zob: u64 = 0;
        let mut pawn_key: u64 = 0;
        let mut b: BitBoard = self.get_occupied();
        while b.is_not_empty() {
            let sq: SQ = b.pop_lsb();
            let piece = self.piece_at_sq(sq).unwrap();
            let key = self.magic_helper.z_piece_at_sq(piece, sq);
            zob ^= key;
            if piece == Piece::P {
                pawn_key ^= key;
            }
        }

        let ep = self.state.ep_square;
        if ep != NO_SQ && ep.is_okay() {
            zob ^= self.magic_helper.z_ep_file(ep);
        }

        match self.turn {
            Player::Black => zob ^= self.magic_helper.z_side(),
            Player::White => {}
        };
        let state =  Arc::get_mut(&mut self.state).unwrap();

        state.zobrast = zob;
        state.pawn_key = pawn_key;
    }
}

// General information

impl Board {
    /// Get the Player whose turn it is to move.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.turn(), Player::White);
    /// ```
    #[inline(always)]
    pub fn turn(&self) -> Player {
        self.turn
    }

    /// Return the Zobrist Hash of the board.
    #[inline(always)]
    pub fn zobrist(&self) -> u64 {
        self.state.zobrast
    }

    /// Return the pawn key of the board.
    ///
    /// This is a semi-unique key for any configuration of pawns on the board.
    #[inline(always)]
    pub fn pawn_key(&self) -> u64 {
        self.state.pawn_key
    }

    /// Get the total number of moves played.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let mut chessboard = Board::default();
    /// assert_eq!(chessboard.moves_played(), 0);
    ///
    /// let moves = chessboard.generate_moves();
    /// chessboard.apply_move(moves[0]);
    /// assert_eq!(chessboard.moves_played(), 1);
    /// ```
    #[inline(always)]
    pub fn moves_played(&self) -> u16 {
        self.half_moves
    }

    /// Get the current depth (half moves from a [Board::shallow_clone()].
    #[inline(always)]
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// Get the number of half-moves since a Pawn Push, castle, or capture.
    #[inline(always)]
    pub fn rule_50(&self) -> i16 {
        self.state.rule_50
    }

    /// Return the Piece, if any, that was last captured.
    #[inline(always)]
    pub fn piece_captured_last_turn(&self) -> Option<Piece> {
        self.state.captured_piece
    }

    /// Get a reference to the MagicHelper pre-computed BitBoards.
    pub fn magic_helper(&self) -> &'static MagicHelper {
        &MAGIC_HELPER
    }

    /// Get the current ply of the board.
    #[inline(always)]
    pub fn ply(&self) -> u16 {
        self.state.ply
    }

    /// Get the current square of en_passant.
    ///
    /// If the current en-passant square is none, it should return 64.
    #[inline(always)]
    pub fn ep_square(&self) -> SQ {
        self.state.ep_square
    }

    /// Gets the BitBoard of all pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,BitBoard};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.get_occupied().0, 0xFFFF00000000FFFF);
    /// ```
    #[inline(always)]
    pub fn get_occupied(&self) -> BitBoard {
        self.occ_all
    }

    /// Get the BitBoard of the squares occupied by the given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.get_occupied_player(Player::White).0, 0x000000000000FFFF);
    /// ```
    #[inline(always)]
    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        self.occ[player as usize]
    }

    /// Returns a Bitboard consisting of only the squares occupied by the White Player.
    #[inline(always)]
    pub fn occupied_white(&self) -> BitBoard {
        self.occ[Player::White as usize]
    }

    /// Returns a BitBoard consisting of only the squares occupied by the Black Player.
    #[inline(always)]
    pub fn occupied_black(&self) -> BitBoard {
        self.occ[Player::Black as usize]
    }

    /// Returns BitBoard of a single player and that one type of piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    /// use pleco::{Player,Piece};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.piece_bb(Player::White,Piece::P).0, 0x000000000000FF00);
    /// ```
    #[inline]
    pub fn piece_bb(&self, player: Player, piece: Piece) -> BitBoard {
        self.bit_boards[player as usize][piece as usize]
    }

    /// Returns the BitBoard of the Queens and Rooks of a given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.sliding_piece_bb(Player::White).count_bits(), 3);
    /// ```
    #[inline]
    pub fn sliding_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::R as usize] ^
            self.bit_boards[player as usize][Piece::Q as usize]
    }
    /// Returns the BitBoard of the Queens and Bishops of a given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.diagonal_piece_bb(Player::White).count_bits(), 3);
    /// ```
    #[inline]
    pub fn diagonal_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::B as usize] ^
            self.bit_boards[player as usize][Piece::Q as usize]
    }

    /// Returns the combined BitBoard of both players for a given piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Piece};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.piece_bb_both_players(Piece::P).0, 0x00FF00000000FF00);
    /// ```
    #[inline(always)]
    pub fn piece_bb_both_players(&self, piece: Piece) -> BitBoard {
        self.bit_boards[Player::White as usize][piece as usize] ^
            self.bit_boards[Player::Black as usize][piece as usize]
    }

    /// Returns the combined BitBoard of both players for two pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Piece,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.piece_two_bb_both_players(Piece::Q,Piece::K).count_bits(), 4);
    /// ```
    #[inline]
    pub fn piece_two_bb_both_players(&self, piece: Piece, piece2: Piece) -> BitBoard {
        self.piece_bb_both_players(piece) | self.piece_bb_both_players(piece2)
    }

    /// Returns the `BitBoard` containing the locations of two given types of pieces for the given
    /// player.
    #[inline(always)]
    pub fn piece_two_bb(&self, piece: Piece, piece2: Piece, player: Player) -> BitBoard {
        self.bit_boards[player as usize][piece as usize] | self.bit_boards[player as usize][piece2 as usize]
    }

    /// Get the total number of pieces of the given piece and player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,Piece};
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.count_piece(Player::White, Piece::P), 8);
    /// ```
    #[inline(always)]
    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        self.piece_counts[player as usize][piece as usize]
    }

    /// Get the total number of pieces a given player has.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,Piece};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.count_pieces_player(Player::White), 16);
    /// ```
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        self.piece_counts[player as usize].iter().sum()
    }

    /// Get the total number of pieces on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,Piece};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::default();
    /// assert_eq!(chessboard.count_all_pieces(), 32);
    /// ```
    #[inline]
    pub fn count_all_pieces(&self) -> u8 {
        self.count_pieces_player(Player::White) + self.count_pieces_player(Player::Black)
    }

    /// Returns the Piece, if any, at the square.
    ///
    /// # Panics
    ///
    /// Panics if the square is not a legal square.
    #[inline]
    pub fn piece_at_sq(&self, sq: SQ) -> Option<Piece> {
        assert!(sq.is_okay());
        self.piece_locations.piece_at(sq)
    }

    /// Returns the Player, if any, occupying a square.
    ///
    /// # Panics
    ///
    /// Panics if the square is not a legal square.
    #[inline]
    pub fn color_of_sq(&self, sq: SQ) -> Option<Player> {
        assert!(sq.is_okay());
        self.piece_locations.player_at(sq)
    }

    /// Returns the player, if any, at the square.
    #[inline(always)]
    pub fn player_at_sq(&self, s: SQ) -> Option<Player> {
        self.piece_locations.player_at(s)
    }

    /// Returns the square of the King for a given player.
    #[inline(always)]
    pub fn king_sq(&self, player: Player) -> SQ {
        (self.bit_boards[player as usize][Piece::K as usize]).to_sq()
    }

    /// Returns the pinned pieces of the given player.
    ///
    /// Pinned is defined as pinned to the same players king
    #[inline(always)]
    pub fn pinned_pieces(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }

    /// Returns the pinned pieces for a given players king. Can contain piece of from both players,
    /// but all are guaranteed to be pinned to the given player's king.
    #[inline(always)]
    pub fn all_pinned_pieces(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize]
    }

    /// Returns the pinning pieces of a given player.
    /// e.g, pieces that are pinning a piece to the opponent's king. This will return the pinned
    /// pieces of both players, pinned to the given player's king.
    #[inline(always)]
    pub fn pinning_pieces(&self, player: Player) -> BitBoard {
        self.state.pinners_king[player as usize]
    }

    /// Return if a player has the possibility of castling for a given CastleType.
    /// This does not ensure a castling is possible for the player, just that the player
    /// has the castling-right available.
    #[inline(always)]
    pub fn can_castle(&self, player: Player, castle_type: CastleType) -> bool {
        self.state.castling.castle_rights(player, castle_type)
    }

    #[inline(always)]
    pub fn player_can_castle(&self, player: Player) -> Castling {
        self.state.castling.player_can_castle(player)
    }

    /// Check if the castle path is impeded for the current player. Does not assume that the
    /// current player has the ability to castle, whether by having the castling-rights to, or
    /// having the rook and king be in the correct square.
    #[inline]
    pub fn castle_impeded(&self, castle_type: CastleType) -> bool {
        let path: BitBoard = BitBoard(CASTLING_PATH[self.turn as usize][castle_type as usize]);
        (path & self.occ_all).is_not_empty()
    }

    /// Square of the Rook that is involved with the current player's castle.
    #[inline]
    pub fn castling_rook_square(&self, castle_type: CastleType) -> SQ {
        SQ(CASTLING_ROOK_START[self.turn as usize][castle_type as usize])
    }

    /// Return the last move played, if any.
    #[inline(always)]
    pub fn last_move(&self) -> Option<BitMove> {
        if self.state.prev_move.is_null() {
            None
        } else {
            Some(self.state.prev_move)
        }
    }

    /// Returns if the current player has castled ever.
    #[inline]
    pub fn has_castled(&self, player: Player) -> bool {
        self.state.castling.has_castled(player)
    }

    /// Returns if the piece (if any) that was captured last move. This method does not
    /// distinguish between not having any last move played and not having a piece last captured.
    #[inline(always)]
    pub fn piece_last_captured(&self) -> Option<Piece> {
        self.state.captured_piece
    }

    //  ------- CHECKING  -------

    /// Returns if current side to move is in check.
    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.state.checkers_bb.is_not_empty()
    }

    /// Return if the current side to move is in check mate.
    ///
    /// This method can be computationally expensive, do not use outside of Engines.
    pub fn checkmate(&self) -> bool {
        self.in_check() && self.generate_moves().is_empty()
    }

    /// Returns if the current side to move is in stalemate.
    ///
    /// This method can be computationally expensive, do not use outside of Engines.
    pub fn stalemate(&self) -> bool {
        !self.in_check() && (self.generate_moves().is_empty() || self.state.rule_50 >= 50)
    }

    /// Return the `BitBoard` of all checks on the current player's king. If the current side
    /// to move is not in check, the `BitBoard` will be empty.
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.state.checkers_bb
    }

    /// Returns the `BitBoard` of pieces the current side can move to discover check.
    /// Discovered check candidates are pieces for the current side to move, that are currently
    /// blocking a check from another piece of the same color.
    #[inline(always)]
    pub fn discovered_check_candidates(&self) -> BitBoard {
        self.state.blockers_king[self.turn.other_player() as usize] &
            self.get_occupied_player(self.turn)
    }

    /// Gets the Pinned pieces for the given player. A pinned piece is defined as a piece that
    /// if suddenly removed, the player would find itself in check.
    #[inline(always)]
    pub fn pieces_pinned(&self, player: Player) -> BitBoard {
        // TODO: combine with Board::piece_pinned
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }
    /// Returns a BitBoard of possible attacks / defends to a square with a given occupancy.
    /// Includes pieces from both players.
    pub fn attackers_to(&self, sq: SQ, occupied: BitBoard) -> BitBoard {
        (self.magic_helper.pawn_attacks_from(sq, Player::Black) &
            self.piece_bb(Player::White, Piece::P)) |
            (self.magic_helper.pawn_attacks_from(sq, Player::White) &
                self.piece_bb(Player::Black, Piece::P)) |
            (self.magic_helper.knight_moves(sq) & self.piece_bb_both_players(Piece::N)) |
            (self.magic_helper.rook_moves(occupied, sq) &
                (self.sliding_piece_bb(Player::White) | self.sliding_piece_bb(Player::Black))) |
            (self.magic_helper.bishop_moves(occupied, sq) &
                (self.diagonal_piece_bb(Player::White) | self.diagonal_piece_bb(Player::Black))) |
            (self.magic_helper.king_moves(sq) & self.piece_bb_both_players(Piece::K))
    }

//  ------- Move Testing -------

    /// Tests if a given move is a legal. This is mostly for checking the legality of moves that
    /// were generated in a pseudo-legal fashion. Generating moves like this is faster, but doesn't
    /// guarantee legality due to the possibility of a discovered check happening.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    pub fn legal_move(&self, m: BitMove) -> bool {
        if m.get_src() == m.get_dest() {
            return false;
        }
        let them: Player = self.turn.other_player();
        let src: SQ = m.get_src();
        let src_bb: BitBoard = src.to_bb();
        let dst: SQ = m.get_dest();

        // Special en_passant case
        if m.move_type() == MoveType::EnPassant {
            let k_sq: SQ = self.king_sq(self.turn);
            let dst_bb: BitBoard = dst.to_bb();
            let captured_sq: SQ = SQ((dst.0 as i8).wrapping_sub(self.turn.pawn_push()) as u8);
            let occupied: BitBoard = (self.get_occupied() ^ src_bb ^ captured_sq.to_bb()) |
                dst_bb;

            return (self.magic_helper.rook_moves(occupied, k_sq) &
                self.sliding_piece_bb(them)).is_empty() &&
                (self.magic_helper.queen_moves(occupied, k_sq) & self.diagonal_piece_bb(them)).is_empty();
        }

        // If Moving the king, check if the square moved to is not being attacked
        // Castles are checked during move gen for check, so we're good there.
        let piece = self.piece_at_sq(src);
        if piece.is_none() {
            return false;
        }

        if piece.unwrap() == Piece::K {
            return m.move_type() == MoveType::Castle ||
                (self.attackers_to(dst, self.get_occupied()) & self.get_occupied_player(them)).is_empty();
        }

        // Making sure not moving a pinned piece
        (self.pinned_pieces(self.turn) & src_bb).is_empty() ||
            self.magic_helper.aligned(src, dst, self.king_sq(self.turn))
    }

    #[doc(hidden)]
    pub fn pseudo_legal_move(&self, _m: BitMove) -> bool {
        unimplemented!()
        // TODO: create pseduo-legal-move
    }

    /// Returns if a move gives check to the opposing player's King.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    pub fn gives_check(&self, m: BitMove) -> bool {
        // I am too drunk to be making this right now
        let src: SQ = m.get_src();
        let dst: SQ = m.get_dest();
        let src_bb: BitBoard = src.to_bb();
        let dst_bb: BitBoard = dst.to_bb();
        let opp_king_sq: SQ = self.king_sq(self.turn.other_player());

        // Stupidity Checks
        assert_ne!(src, dst);
        assert_eq!(self.color_of_sq(src).unwrap(), self.turn);

        // Searches for direct checks from the pre-computed array
        if (self.state.check_sqs[self.piece_at_sq(src).unwrap() as usize] & dst_bb).is_not_empty() {
            return true;
        }

        // Discovered (Indirect) checks, where a sniper piece is attacking the king
        if (self.discovered_check_candidates() & src_bb).is_not_empty()  // check if the piece is blocking a sniper
            && !self.magic_helper.aligned(src, dst, opp_king_sq) { // Make sure the dst square is not aligned
            return true;
        }

        match m.move_type() {
            MoveType::Normal => false, // Nothing to check here
            MoveType::Promotion => {
                // check if the Promo Piece attacks king
                let attacks_bb = match m.promo_piece() {
                    Piece::N => self.magic_helper.knight_moves(dst),
                    Piece::B => {
                        self.magic_helper
                            .bishop_moves(self.get_occupied() ^ src_bb, dst)
                    }
                    Piece::R => {
                        self.magic_helper
                            .rook_moves(self.get_occupied() ^ src_bb, dst)
                    }
                    Piece::Q => {
                        self.magic_helper
                            .queen_moves(self.get_occupied() ^ src_bb, dst)
                    }
                    _ => unreachable!(),
                };
                (attacks_bb & opp_king_sq.to_bb()).is_not_empty()
            }
            MoveType::EnPassant => {
                // Check for indirect check from the removal of the captured pawn
                let captured_sq: SQ = SQ::make(dst.file(), src.rank());
                let b: BitBoard = (self.get_occupied() ^ src_bb ^ captured_sq.to_bb()) | dst_bb;

                let turn_sliding_p: BitBoard = self.sliding_piece_bb(self.turn);
                let turn_diag_p: BitBoard = self.diagonal_piece_bb(self.turn);

                ((self.magic_helper.rook_moves(b, opp_king_sq) & turn_sliding_p) |
                    (self.magic_helper.bishop_moves(b, opp_king_sq) & turn_diag_p)).is_not_empty()
            }
            MoveType::Castle => {
                // Check if the rook attacks the King now
                let k_from: SQ = src;
                let r_from: SQ = dst;

                let k_to: SQ = self.turn.relative_square( { if r_from > k_from { SQ(6) } else { SQ(2) } });
                let r_to: SQ = self.turn.relative_square( { if r_from > k_from { SQ(5) } else { SQ(3) } });

                let opp_k_bb = opp_king_sq.to_bb();
                (self.magic_helper.rook_moves(BitBoard(0), r_to) & opp_k_bb).is_not_empty() &&
                    (self.magic_helper.rook_moves(
                        r_to.to_bb() | k_to.to_bb() |
                            (self.get_occupied() ^ k_from.to_bb() ^ r_from.to_bb()),
                        r_to,
                    ) & opp_k_bb).is_not_empty()
            }
        }
    }

    /// Returns the piece that was moved from a given BitMove.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    #[inline]
    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
        self.piece_at_sq(src).unwrap() // panics if no piece here :)
    }

    /// Returns the piece that was captured, if any from a given BitMove.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    #[inline]
    pub fn captured_piece(&self, m: BitMove) -> Option<Piece> {
        if m.is_en_passant() {
            return Some(Piece::P);
        }
        let dst = m.get_dest();
        self.piece_at_sq(dst)
    }

    /// Returns a prettified String of the current `Board`, for easy command line displaying.
    ///
    /// Capital Letters represent white pieces, while lower case represents black pieces.
    pub fn pretty_string(&self) -> String {
        let mut s = String::with_capacity(SQ_CNT * 2 + 8);
        for sq in SQ_DISPLAY_ORDER.iter() {
            let op = self.piece_locations.player_piece_at(SQ(*sq));
            let char = if op.is_some() {
                let player = op.unwrap().0;
                let piece = op.unwrap().1;
                PIECE_DISPLAYS[player as usize][piece as usize]
            } else {
                '-'
            };
            s.push(char);
            s.push(' ');
            if sq % 8 == 7 {
                s.push('\n');
            }
        }
        s
    }

    /// Returns a clone of the current `PieceLocations`.
    pub fn get_piece_locations(&self) -> PieceLocations {
        self.piece_locations.clone()
    }

    /// Get Debug Information.
    pub fn print_debug_info(&self) {
        println!("White Pinners ");
        println!("{}", self.state.pinners_king[0]);
        println!("Black Pinners ");
        println!("{}", self.state.pinners_king[1]);

        println!("White Blockers ");
        println!("{}", self.state.blockers_king[0]);
        println!("Black Blockers ");
        println!("{}", self.state.blockers_king[1]);

        println!("Checkers ");
        println!("{}", self.state.checkers_bb);

        println!("Bishop check sqs");
        println!("{}", self.state.check_sqs[Piece::B as usize]);

        println!("Rook check sqs");
        println!("{}", self.state.check_sqs[Piece::R as usize]);

        println!("Queen check sqs");
        println!("{}", self.state.check_sqs[Piece::Q as usize]);
    }

    /// Prints a prettified representation of the board.
    pub fn pretty_print(&self) {
        println!("{}", self.pretty_string());
    }

    /// Print the board alongside useful information.
    ///
    /// Mostly for Debugging useage.
    pub fn fancy_print(&self) {
        self.pretty_print();
        println!(
            "Castling bits: {:b}, Rule 50: {}, ep_sq: {}",
            self.state.castling,
            self.state.rule_50,
            self.state.ep_square
        );
        println!(
            "Total Moves: {}, ply: {}, depth: {}",
            self.half_moves,
            self.state.ply,
            self.depth
        );
        println!("Zobrist: {:x}", self.state.zobrast);
        println!();


    }

}

// TODO: Error Propagation

/// Represents possible Errors encountered while building a `Board` from a fen string.
#[derive(Fail, Debug)]
pub enum BoardError {
    #[fail(display = "incorrect number of kings for {}: {}", player, num)]
    IncorrectKingNum {player: Player, num: u8},
}

impl Board {
    /// Checks the basic status of the board, returning false if something is wrong.
    pub fn is_ok_quick(&self) -> bool {
        self.piece_at_sq(self.king_sq(Player::White)).unwrap() == Piece::K
            && self.piece_at_sq(self.king_sq(Player::Black)).unwrap() == Piece::K
            && (self.state.ep_square == NO_SQ
            || self.turn.relative_rank_of_sq(self.state.ep_square) == Rank::R6)
    }

    /// Checks if the current state of the Board is okay.
    pub fn is_okay(&self) -> Result<(), BoardError> {
        self.check_king()?;
        Ok(())
    }

    fn check_king(&self) -> Result<(), BoardError> {
        // TODO: Implement attacks to opposing king must be zero
        let w_king_num = self.count_piece(Player::White, Piece::K);
        let b_king_num = self.count_piece(Player::Black, Piece::K);
        if w_king_num != 1 { return Err(BoardError::IncorrectKingNum {player: Player::White, num: w_king_num}); }
        if w_king_num != 1 { return Err(BoardError::IncorrectKingNum {player: Player::Black, num: b_king_num}); }
        Ok(())
    }
//
//    fn check_bitboards(&self) -> bool {
//        assert_eq!(self.occupied_white() & self.occupied_black(), BitBoard(0));
//        assert_eq!(
//            self.occupied_black() | self.occupied_white(),
//            self.get_occupied()
//        );
//
//        let all: BitBoard = self.piece_bb(Player::White, Piece::P) ^ self.piece_bb(Player::Black, Piece::P)
//            ^ self.piece_bb(Player::White, Piece::N) ^ self.piece_bb(Player::Black, Piece::N)
//            ^ self.piece_bb(Player::White, Piece::B) ^ self.piece_bb(Player::Black, Piece::B)
//            ^ self.piece_bb(Player::White, Piece::R) ^ self.piece_bb(Player::Black, Piece::R)
//            ^ self.piece_bb(Player::White, Piece::Q) ^ self.piece_bb(Player::Black, Piece::Q)
//            ^ self.piece_bb(Player::White, Piece::K) ^ self.piece_bb(Player::Black, Piece::K);
//        // Note, this was once all.0, self.get_occupied.0
//        assert_eq!(all, self.get_occupied());
//        true
//    }

//    fn check_state_info(&self) -> bool {
//        true
//    }
//
//    fn check_lists(&self) -> bool {
//        true
//    }
//
//    fn check_castling(&self) -> bool {
//        true
//    }
}

#[derive(Eq, PartialEq)]
enum RandGen {
    InCheck,
    NoCheck,
    All
}

/// Random [`Board`] generator. Creates either one or many random boards with optional
/// parameters.
///
/// # Examples
///
/// Create one [`Board`] with at least 5 moves played that is created in a pseudo-random
/// fashion.
///
/// ```
/// use pleco::board::{Board,RandBoard};
///
/// let rand_boards: Board = RandBoard::new()
///     .pseudo_random(12455)
///     .min_moves(5)
///     .one();
/// ```
///
/// Create a `Vec` of 10 random [`Board`]s that are guaranteed to not be in check.
///
/// ```
/// use pleco::board::{Board,RandBoard};
///
/// let rand_boards: Vec<Board> = RandBoard::new()
///     .pseudo_random(12455)
///     .no_check()
///     .many(10);
/// ```
///
/// [`Board`]: struct.Board.html
pub struct RandBoard {
    gen_type: RandGen,
    minimum_move: u16,
    favorable_player: Player,
    prng: PRNG,
    seed: u64
}

impl Default for RandBoard {
    fn default() -> Self {
        RandBoard {
            gen_type: RandGen::All,
            minimum_move: 2,
            favorable_player: Player::Black,
            prng: PRNG::init(1),
            seed: 0
        }
    }
}

impl RandBoard {
    /// Create a new `RandBoard` object.
    pub fn new() -> Self {
        RandBoard {
            gen_type: RandGen::All,
            minimum_move: 1,
            favorable_player: Player::Black,
            prng: PRNG::init(1),
            seed: 0
        }
    }

    /// Creates a `Vec<Board>` full of `Boards` containing random positions. The
    /// `Vec` will be of size 'size'.
    pub fn many(mut self, size: usize) -> Vec<Board> {
        let mut boards: Vec<Board> = Vec::with_capacity(size);
        for _x in 0..size {
            boards.push(self.go());
        };
        boards
    }

    /// Creates a singular `Board` with a random position.
    pub fn one(mut self) -> Board {
        self.go()
    }

    /// Turns PseudoRandom generation on. This allows for the same random `Board`s
    /// to be created from the same seed.
    pub fn pseudo_random(mut self, seed: u64) -> Self {
        self.seed = if seed == 0 {1} else {seed};
        self.prng = PRNG::init(seed);
        self
    }

    /// Sets the minimum moves a randomly generated `Board` must contain.
    pub fn min_moves(mut self, moves: u16) -> Self {
        self.minimum_move = moves;
        self
    }

    /// Garuntees that the boards returned are only in check,
    pub fn in_check(mut self) -> Self {
        self.gen_type = RandGen::InCheck;
        self
    }

    /// Garuntees that the boards returned are not in check.
    pub fn no_check(mut self) -> Self {
        self.gen_type = RandGen::NoCheck;
        self
    }

    /// This makes a board.
    fn go(&mut self) -> Board {
        self.favorable_player = if self.random() % 2 == 0 {
            Player::White
        } else {
            Player::Black
        };
        loop {
            let mut board = Board::default();
            let mut iterations = 0;
            let mut moves = board.generate_moves();

            while iterations < 100 && !moves.is_empty() {
                let mut rand = self.random() % max(90 - min(max(iterations, 0),90), 13);
                if iterations > 20 {
                    rand %= 60;
                    if iterations > 36 {
                        rand >>= 1;
                    }
                }

                if rand == 0 && self.to_ret(&board){
                   return board;
                }

                self.apply_random_move(&mut board);
                moves = board.generate_moves();
                iterations += 1;
            }
        }

    }

    /// Creates a random number.
    fn random(&mut self) -> usize {
        if self.seed == 0 {
            return rand::random::<usize>();
        }
        self.prng.rand() as usize
    }

    fn to_ret(&self, board: &Board) -> bool {
        let gen: bool =match self.gen_type {
            RandGen::All => true,
            RandGen::InCheck => board.in_check(),
            RandGen::NoCheck => !board.in_check()
        };
        gen && (board.moves_played() >= self.minimum_move)
    }

    fn apply_random_move(&mut self, board: &mut Board) {
        let (rand_num, favorable): (usize, bool) =
            if self.favorable(board.turn) {
                (24, true)
            } else {
                (14, false)
            };

        let best_move = if self.random() % rand_num == 0 {
            let moves = board.generate_moves();
            moves[self.random() % moves.len()]
        } else if self.random() % 5 == 0 {
            JamboreeSearcher::best_move_depth(board.shallow_clone(),2)
        } else if self.random() % 3 == 0 {
            JamboreeSearcher::best_move_depth(board.shallow_clone(),3)
        } else if !favorable && self.random() % 4 < 3 {
            JamboreeSearcher::best_move_depth(board.shallow_clone(),3)
        } else {
            IterativeSearcher::best_move_depth(board.shallow_clone(),4)
        };
        board.apply_move(best_move);
    }

    fn favorable(&self, player: Player) -> bool {
        self.gen_type == RandGen::InCheck
            && self.favorable_player == player
    }
}


#[cfg(test)]
mod tests {

    extern crate rand;
    use board::Board;

    #[test]
    fn random_move_apply() {
        let mut board = Board::default();
        let mut ply = 1000;
        while ply > 0 && !board.checkmate() && !board.stalemate() {
            let moves = board.generate_moves();
            let picked_move = moves[rand::random::<usize>() % moves.len()];
            board.apply_move(picked_move);
            ply -= 1;
        }
    }

    #[test]
    fn fen_equality() {
        let mut board = Board::default();
        let mut ply = 1000;
        let mut fen_stack = Vec::new();
        while ply > 0 && !board.checkmate() && !board.stalemate() {
            fen_stack.push(board.get_fen());
            let moves = board.generate_moves();
            let picked_move = moves[rand::random::<usize>() % moves.len()];
            board.apply_move(picked_move);
            ply -= 1;
        }

        while !fen_stack.is_empty() {
            board.undo_move();
            assert_eq!(board.get_fen(),fen_stack.pop().unwrap());
        }
    }

    #[test]
    fn zob_equality() {
        let mut board = Board::default();
        let mut ply = 1000;
        let mut zobrist_stack = Vec::new();
        while ply > 0 && !board.checkmate() && !board.stalemate() {
            zobrist_stack.push(board.zobrist());
            let moves = board.generate_moves();
            let picked_move = moves[rand::random::<usize>() % moves.len()];
            board.apply_move(picked_move);
            ply -= 1;
        }

        while !zobrist_stack.is_empty() {
            board.undo_move();
            assert_eq!(board.zobrist(),zobrist_stack.pop().unwrap());
        }
    }

    #[test]
    fn rand_board_gen_one() {
        let boards_1 = Board::random()
            .pseudo_random(550087423)
            .min_moves(3)
            .one();

        let boards_2 = Board::random()
            .pseudo_random(550087423)
            .min_moves(3)
            .one();

        assert_eq!(boards_1, boards_2);
    }

    #[test]
    fn rand_board_gen_many() {
        let mut boards_1 = Board::random()
            .pseudo_random(222227835)
            .many(5);

        let mut boards_2 = Board::random()
            .pseudo_random(222227835)
            .many(5);

        assert_eq!(boards_1.len(),boards_2.len());
        while !boards_1.is_empty() {
            assert_eq!(boards_1.pop(), boards_2.pop());
        }
    }
}