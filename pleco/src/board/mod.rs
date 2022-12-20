//! This module contains `Board`, the object representing the current state of a chessboard.
//! All modifications to the current state of the board is done through this object, as well as
//! gathering information about the current state of the board.
//!
//! This module also contains structures used by the board, such as [`CastlingRights`] for
//! determining castling rights throughout a game. Other utilities that may be of use
//! are [`PieceLocations`], which maps squares on a chessboard to pieces and players.
//!
//! [`CastlingRights`]: castle_rights/struct.Castling.html
//! [`PieceLocations`]: piece_locations/struct.Eval.html

use std::cmp::{max, min, PartialEq};
use std::hint::unreachable_unchecked;
use std::option::*;
use std::{char, fmt, num};

use rand;

use bot_prelude::AlphaBetaSearcher;
use core::bitboard::BitBoard;
use core::masks::*;
use core::mono_traits::*;
use core::move_list::{MoveList, ScoringMoveList};
use core::piece_move::{BitMove, MoveType};
use core::score::*;
use core::sq::{NO_SQ, SQ};
use core::*;
use helper::prelude::*;
use helper::Helper;
use tools::pleco_arc::{Arc, UniqueArc};
use tools::prng::PRNG;
use tools::{PreFetchable, Searcher};

use self::board_state::BoardState;
use self::castle_rights::Castling;
use self::movegen::{Legal, MoveGen, PseudoLegal};
use self::piece_locations::PieceLocations;

pub mod board_state;
pub mod castle_rights;
pub mod fen;
pub mod movegen;
pub mod perft;
mod pgn;
pub mod piece_locations;

/// Represents possible Errors encountered while building a `Board` from a fen string.
pub enum FenBuildError {
    NotEnoughSections {
        sections: usize,
    },
    IncorrectRankAmounts {
        ranks: usize,
    },
    UnrecognizedTurn {
        turn: String,
    },
    EPSquareUnreadable {
        ep: String,
    },
    EPSquareInvalid {
        ep: String,
    },
    SquareSmallerRank {
        rank: usize,
        square: String,
    },
    SquareLargerRank {
        rank: usize,
        square: String,
    },
    UnrecognizedPiece {
        piece: char,
    },
    UnreadableMoves(num::ParseIntError),
    IllegalNumCheckingPieces {
        num: u8,
    },
    IllegalCheckState {
        piece_1: PieceType,
        piece_2: PieceType,
    },
    TooManyPawns {
        player: Player,
        num: u8,
    },
    PawnOnLastRow,
}

impl From<num::ParseIntError> for FenBuildError {
    fn from(err: num::ParseIntError) -> FenBuildError {
        FenBuildError::UnreadableMoves(err)
    }
}

impl fmt::Debug for FenBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FenBuildError::NotEnoughSections { sections } => writeln!(
                f,
                "invalid number of fen sections: {}, expected 6",
                sections
            ),
            FenBuildError::IncorrectRankAmounts { ranks } => {
                writeln!(f, "invalid number of ranks: {}, expected 8", ranks)
            }
            FenBuildError::UnrecognizedTurn { ref turn } => {
                writeln!(f, "invalid turn: {}, expected 'w' or 'b'", turn)
            }
            FenBuildError::EPSquareUnreadable { ref ep } => {
                writeln!(f, "unreadable En-passant square: {}", ep)
            }
            FenBuildError::EPSquareInvalid { ref ep } => {
                writeln!(f, "invalid En-passant square: {}", ep)
            }
            FenBuildError::SquareSmallerRank { rank, ref square } => writeln!(
                f,
                "square number too small for rank, rank: {} square: {},",
                rank, square
            ),
            FenBuildError::SquareLargerRank { rank, ref square } => writeln!(
                f,
                "square number too large for rank, rank: {} square: {},",
                rank, square
            ),
            FenBuildError::UnrecognizedPiece { piece } => {
                writeln!(f, "unrecognized piece: {}", piece)
            }
            FenBuildError::UnreadableMoves(ref err) => {
                writeln!(f, "An unknown error has occurred {:?}", err)
            }
            FenBuildError::IllegalNumCheckingPieces { num } => {
                writeln!(f, "too many checking piece: {}", num)
            }
            FenBuildError::IllegalCheckState { piece_1, piece_2 } => writeln!(
                f,
                "these two pieces cannot check the king at the same time: {}, {}",
                piece_1, piece_2
            ),
            FenBuildError::TooManyPawns { player, num } => writeln!(
                f,
                "Too many pawns for player: player: {}, # pawns {}",
                player, num
            ),
            FenBuildError::PawnOnLastRow => writeln!(f, "Pawn on first or last row"),
        }
    }
}

struct PreFetchDummy {}

impl PreFetchable for PreFetchDummy {
    fn prefetch(&self, _key: u64) {}
}

/// Represents a Chessboard through a `Board`.
///
/// Board contains everything that needs to be known about the current state of the Game. It is used
/// by both Engines and Players / Bots alike.
///
/// Ideally, the Engine contains the original Representation of a board (owns the board), and utilizes
/// `Board::shallow_clone()` to share this representation with Players.
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
    turn: Player,                                     // Current turn
    bbs: [BitBoard; PIECE_TYPE_CNT],                  // Occupancy per player per piece
    bbs_player: [BitBoard; PLAYER_CNT],               // Occupancy per Player
    half_moves: u16,                                  // Total moves played
    depth: u16,                                       // Current depth since last shallow_copy
    piece_counts: [[u8; PIECE_TYPE_CNT]; PLAYER_CNT], // Count of each Piece
    piece_locations: PieceLocations,                  // Mapping Squares to Pieces and Plauers
    zobrist_history: Vec<u64>,                        // Historic Zobrist keys of the board
    threefold_repetition: bool,                       // Whether the board has been repeated 3 times

    // State of the Board, Un modifiable.
    // Arc to allow easy and quick copying of boards without copying memory
    // or recomputing BoardStates.
    state: Arc<BoardState>,

    /// Reference to the pre-computed lookup tables.
    #[doc(hidden)]
    pub magic_helper: Helper,
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
        self.turn == other.turn
            && self.bbs[PieceType::All as usize] == other.bbs[PieceType::All as usize]
            && *self.state == *other.state
            && self.piece_locations == other.piece_locations
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        self.shallow_clone()
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::start_pos()
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
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.count_pieces_player(Player::White),16);
    /// ```
    pub fn start_pos() -> Board {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
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
    /// let mut chessboard = Board::start_pos();
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
            bbs: self.bbs,
            bbs_player: self.bbs_player,
            half_moves: self.half_moves,
            depth: 0,
            piece_counts: self.piece_counts,
            piece_locations: self.piece_locations.clone(),
            state: Arc::clone(&self.state),
            magic_helper: self.magic_helper,
            zobrist_history: self.zobrist_history.clone(),
            threefold_repetition: self.threefold_repetition,
        }
    }

    /// Constructs a parallel clone of the Board.
    ///
    /// Similar to `Board::shallow_clone()`, but keeps the current search depth the same.
    /// Should be used when implementing a searcher, and want to search a list of moves
    /// in parallel with different threads.
    ///
    /// # Safety
    ///
    /// After this method has called, `Board::undo_move()` cannot be called immediately after.
    /// Undoing moves can only be done once a move has been played, and cannot be called more
    /// times than moves have been played since calling `Board::parallel_clone()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    ///
    /// let mut chessboard = Board::start_pos();
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
            bbs: self.bbs,
            bbs_player: self.bbs_player,
            half_moves: self.half_moves,
            depth: self.depth,
            piece_counts: self.piece_counts,
            piece_locations: self.piece_locations.clone(),
            state: Arc::clone(&self.state),
            magic_helper: self.magic_helper,
            zobrist_history: self.zobrist_history.clone(),
            threefold_repetition: self.threefold_repetition,
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
    ///     .pseudo_random(6622225)
    ///     .min_moves(5)
    ///     .one();
    /// ```
    ///
    /// Create a `Vec` of 3 random `Board`s that are guaranteed to not be in check.
    ///
    /// ```
    /// use pleco::board::{Board,RandBoard};
    ///
    /// let rand_boards: Vec<Board> = Board::random()
    ///     .no_check()
    ///     .many(3);
    /// ```
    pub fn random() -> RandBoard {
        RandBoard::default()
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
    /// let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    /// assert_eq!(board.count_all_pieces(),32);
    ///
    ///
    /// let obviously_not_a_fen = "This shouldn't parse!";
    /// let bad_board = Board::from_fen(obviously_not_a_fen);
    /// assert!(bad_board.is_err());
    /// ```
    ///
    /// # Safety
    ///
    /// The FEN string must be valid, or else the method will return an Error.
    ///
    /// There is a possibility of the FEN string representing an invalid position, with no panics resulting.
    /// The Constructed Board may have some Undefined Behavior as a result. It is up to the user to give a
    /// valid FEN string.
    pub fn from_fen(fen: &str) -> Result<Board, FenBuildError> {
        // split the string by white space
        let det_split: Vec<&str> = fen.split_whitespace().collect();

        // must have 6 parts :
        // [ Piece Placement, Side to Move, Castling Ability, En Passant square, Half moves, full moves]
        if det_split.len() < 4 || det_split.len() > 6 {
            return Err(FenBuildError::NotEnoughSections {
                sections: det_split.len(),
            });
        }

        // Split the first part by '/' for locations
        let b_rep: Vec<&str> = det_split[0].split('/').collect();

        if b_rep.len() != 8 {
            return Err(FenBuildError::IncorrectRankAmounts { ranks: b_rep.len() });
        }

        let piece_loc = PieceLocations::from_partial_fen(b_rep.as_slice())?;

        // Create the Board
        let mut b = Board {
            turn: Player::White,
            bbs: [BitBoard(0); PIECE_TYPE_CNT],
            bbs_player: [BitBoard(0); PLAYER_CNT],
            half_moves: 0,
            depth: 0,
            piece_counts: [[0; PIECE_TYPE_CNT]; PLAYER_CNT],
            piece_locations: PieceLocations::blank(),
            state: Arc::new(BoardState::blank()),
            magic_helper: Helper::new(),
            zobrist_history: Vec::new(),
            threefold_repetition: false,
        };

        for &(sq, plyr, piece) in piece_loc.iter() {
            b.put_piece_c(Piece::make_lossy(plyr, piece), sq);
        }

        // Side to Move
        let turn_char: char =
            det_split[1]
                .chars()
                .next()
                .ok_or(FenBuildError::UnrecognizedTurn {
                    turn: det_split[1].to_string(),
                })?;

        let turn: Player = match turn_char {
            'b' => Player::Black,
            'w' => Player::White,
            _ => {
                return Err(FenBuildError::UnrecognizedTurn {
                    turn: det_split[1].to_string(),
                });
            }
        };

        b.turn = turn;

        // Castle Bytes
        let mut castle_bytes = Castling::empty();
        for ch in det_split[2].chars() {
            castle_bytes.add_castling_char(ch);
        }

        let mut ep_sq: SQ = SQ(0);
        for (i, character) in det_split[3].chars().enumerate() {
            if i > 1 {
                return Err(FenBuildError::EPSquareUnreadable {
                    ep: det_split[3].to_string(),
                });
            }
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
                    _ => {
                        return Err(FenBuildError::EPSquareUnreadable {
                            ep: det_split[3].to_string(),
                        });
                    }
                }
            } else {
                let digit = character
                    .to_digit(10)
                    .ok_or(FenBuildError::EPSquareUnreadable {
                        ep: det_split[3].to_string(),
                    })? as u8;

                // must be 3 or 6
                if digit == 3 {
                    ep_sq += SQ(16); // add two ranks
                } else if digit == 6 {
                    ep_sq += SQ(40);
                } else {
                    return Err(FenBuildError::EPSquareInvalid {
                        ep: det_split[3].to_string(),
                    });
                }
            }
        }

        if ep_sq == SQ(0) {
            ep_sq = NO_SQ
        }

        // rule 50 counts
        let rule_50 = if det_split.len() >= 5 && det_split[4] != "-" {
            det_split[4].parse::<i16>()?
        } else {
            0
        };

        // Total Moves Played
        // Moves is defined as every time White moves, so gotta translate to total moves
        let mut total_moves = if det_split.len() >= 6 && det_split[5] != "-" {
            (det_split[5].parse::<u16>()? - 1) * 2
        } else {
            0
        };

        if turn == Player::Black {
            total_moves += 1
        };

        b.half_moves = total_moves;

        // Set State info
        let b_state = {
            // Set Check info
            let mut state: BoardState = BoardState::blank();
            state.castling = castle_bytes;
            state.rule_50 = rule_50;
            state.ep_square = ep_sq;
            state.set(&b);
            state
        };

        b.state = Arc::new(b_state);

        // validate
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
    /// let board = Board::start_pos();
    /// assert_eq!(board.fen(),"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// ```
    pub fn fen(&self) -> String {
        // TODO: Doesn't display if rank 8 has zero pieces on it
        let mut s = String::default();

        for reverse_rnk in 0..8 {
            let mut blanks = 0;
            let rank = 7 - reverse_rnk;

            if reverse_rnk != 0 {
                s.push('/');
            }

            for file in 0..FILE_CNT {
                let sq = SQ(rank as u8 * 8 + file as u8);
                let piece = self.piece_locations.piece_at(sq);
                if piece != Piece::None {
                    if blanks != 0 {
                        s.push(char::from_digit(blanks, 10).unwrap());
                        blanks = 0;
                    }
                    s.push(piece.character_lossy());
                } else {
                    blanks += 1;
                }
            }
            if blanks != 0 {
                s.push(char::from_digit(blanks, 10).unwrap());
            }
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
    /// The passed in `BitMove` must be a legal move for the current position.
    ///
    /// # Panics
    ///
    /// The supplied BitMove must be both a valid move for that position, as well as a
    /// valid `BitMove`, Otherwise, a panic will occur. Valid BitMoves can be generated with
    /// `Board::generate_moves()`, which guarantees that only Legal moves will be created.
    pub fn apply_move(&mut self, bit_move: BitMove) {
        let gives_check: bool = self.gives_check(bit_move);
        let pt_d = PreFetchDummy {};
        let mt_d = PreFetchDummy {};
        self.apply_move_pft_chk::<PreFetchDummy, PreFetchDummy>(
            bit_move,
            gives_check,
            &pt_d,
            &mt_d,
        );
    }

    /// Applies a move to the Board. This method is only useful if before a move is applied to
    /// a board, the ability of the move to give check is applied. If it is not needed to know
    /// if the move gives check or not, consider using `Board::apply_move` instead.
    ///
    /// This method also takes in two generic parameters implementing `PreFetchable`, one of which
    /// will prefetch from the A table taking in a pawn key, the other of which pre-fetching
    /// from a table utilizing the material key.
    ///
    /// # Safety
    ///
    /// The passed in `BitMove` must be a legal move for the current position, and the gives_check
    /// parameter must be correct for the move.
    ///
    /// # Panics
    ///
    /// The supplied BitMove must be both a valid move for that position, as well as a
    /// valid `BitMove`, Otherwise, a panic will occur. Valid BitMoves can be generated with
    /// `Board::generate_moves()`, which guarantees that only Legal moves will be created.
    ///
    /// The second parameter, `gives_check`, must be true if the move gives check, or false
    /// if the move doesn't give check. If an incorrect `gives_check` is supplied, undefined
    /// behavior will follow.
    pub fn apply_move_pft_chk<PT, MT>(
        &mut self,
        bit_move: BitMove,
        gives_check: bool,
        pawn_table: &PT,
        material_table: &MT,
    ) where
        PT: PreFetchable,
        MT: PreFetchable,
    {
        // Check for stupidity
        assert_ne!(bit_move.get_src(), bit_move.get_dest());

        // Zobrist Hash
        let mut pawn_key: u64 = self.state.pawn_key;
        let mut zob: u64 = self.state.zobrist ^ z_side();
        let mut material_key: u64 = self.state.material_key;

        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = UniqueArc::new(self.state.partial_clone());

        // Separate Block to allow dereferencing the BoardState
        // As there is garunteed only one owner of the Arc, this is allowed
        {
            let new_state: &mut BoardState = &mut next_arc_state;

            // Set the prev state
            new_state.prev = Some(Arc::clone(&self.state));

            // Increment these
            self.half_moves += 1;
            self.depth += 1;
            self.zobrist_history.push(self.state.zobrist);
            new_state.rule_50 += 1;
            new_state.ply += 1;
            new_state.prev_move = bit_move;

            let us = self.turn;
            let them = !us;
            let from: SQ = bit_move.get_src();
            let mut to: SQ = bit_move.get_dest();
            let piece: Piece = self.piece_at_sq(from);

            debug_assert_ne!(piece, Piece::None);

            let captured: Piece = if bit_move.is_en_passant() {
                Piece::make_lossy(them, PieceType::P)
            } else {
                self.piece_at_sq(to)
            };

            // Sanity checks
            assert_eq!(piece.player_lossy(), us);

            if bit_move.is_castle() {
                // Sanity Checks, moved piece should be K, "captured" should be R
                // As this is the encoding of Castling
                assert_eq!(captured.type_of(), PieceType::R);
                assert_eq!(piece.type_of(), PieceType::K);

                let mut r_src: SQ = SQ(0);
                let mut r_dst: SQ = SQ(0);

                // yay helper methods
                self.apply_castling(us, from, &mut to, &mut r_src, &mut r_dst);
                let rook = Piece::make_lossy(us, PieceType::R);
                new_state.psq += psq(rook, r_dst) - psq(rook, r_src);
                zob ^= z_square(r_src, rook) ^ z_square(r_dst, rook);
                new_state.captured_piece = PieceType::None;
            } else if captured != Piece::None {
                let mut cap_sq: SQ = to;
                if captured.type_of() == PieceType::P {
                    if bit_move.is_en_passant() {
                        assert_eq!(cap_sq, self.state.ep_square);
                        match us {
                            Player::White => cap_sq -= SQ(8),
                            Player::Black => cap_sq += SQ(8),
                        };
                        assert_eq!(piece.type_of(), PieceType::P);
                        assert_eq!(us.relative_rank(Rank::R6), to.rank());
                        assert_eq!(self.piece_at_sq(to), Piece::None);
                        assert_eq!(self.piece_at_sq(cap_sq).type_of(), PieceType::P);
                        assert_eq!(self.piece_at_sq(cap_sq).player().unwrap(), them);
                        self.remove_piece_c(captured, cap_sq);
                    } else {
                        self.remove_piece_c(captured, cap_sq);
                    }
                    pawn_key ^= z_square(cap_sq, captured);
                } else {
                    new_state.nonpawn_material[them as usize] -= piece_value(captured, false);
                    self.remove_piece_c(captured, cap_sq);
                }
                zob ^= z_square(cap_sq, captured);

                // update material key and prefetch access to a Material Table
                let cap_count = self.count_piece(them, captured.type_of());
                material_key ^= z_square(SQ(cap_count), captured);
                material_table.prefetch(material_key);
                new_state.psq -= psq(captured, cap_sq);

                // Reset Rule 50
                new_state.rule_50 = 0;
                new_state.captured_piece = captured.type_of();
            }

            // Update hash for moving piece
            zob ^= z_square(to, piece) ^ z_square(from, piece);

            if self.state.ep_square != NO_SQ {
                zob ^= z_ep(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            // Update castling rights
            if !new_state.castling.is_empty()
                && (to.castle_rights_mask() | from.castle_rights_mask()) != 0
            {
                let castle_zob_index = new_state.castling.update_castling(to, from);
                zob ^= z_castle(castle_zob_index);
            }

            // Actually move the piece
            if !bit_move.is_castle() {
                self.move_piece_c(piece, from, to);
            }

            // Pawn Moves need special help :(
            if piece.type_of() == PieceType::P {
                if to.0 ^ from.0 == 16 {
                    // Double Push
                    let poss_ep: u8 = (to.0 as i8 - us.pawn_push()) as u8;

                    // Set en-passant square if the moved pawn can be captured
                    if (pawn_attacks_from(SQ(poss_ep), us) & self.piece_bb(them, PieceType::P))
                        .is_not_empty()
                    {
                        new_state.ep_square = SQ(poss_ep);
                        zob ^= z_ep(new_state.ep_square);
                    }
                } else if bit_move.is_promo() {
                    let promo_piece: PieceType = bit_move.promo_piece();
                    let us_promo = Piece::make_lossy(us, promo_piece);
                    self.remove_piece_c(piece, to);
                    self.put_piece_c(us_promo, to);
                    zob ^= z_square(to, us_promo) ^ z_square(to, piece);

                    // We add the zobrist key for the pawn promotion square as we'll just take
                    // it away later
                    pawn_key ^= z_square(to, piece);

                    let promo_count = self.count_piece(us, promo_piece);
                    let pawn_count = self.count_piece(us, PieceType::P);
                    material_key ^=
                        z_square(SQ(promo_count - 1), us_promo) ^ z_square(SQ(pawn_count), piece);

                    new_state.psq += psq(us_promo, to) - psq(piece, to);
                    new_state.nonpawn_material[us as usize] += piece_value(us_promo, false);
                }

                // update pawn key and prefetch access
                pawn_key ^= z_square(from, piece) ^ z_square(to, piece);
                pawn_table.prefetch2(pawn_key);
                new_state.rule_50 = 0;
            }

            new_state.psq += psq(piece, to) - psq(piece, from);
            new_state.captured_piece = captured.type_of();
            new_state.zobrist = zob;
            new_state.pawn_key = pawn_key;
            new_state.material_key = material_key;

            new_state.checkers_bb = if gives_check {
                self.attackers_to(self.king_sq(them), self.occupied())
                    & self.get_occupied_player(us)
            } else {
                BitBoard(0)
            };

            self.threefold_repetition =
                self.zobrist_history.iter().filter(|&x| x == &zob).count() >= 2;
            self.turn = them;

            // Set the checking information
            new_state.set_check_info(self);
        }
        self.state = next_arc_state.shareable();

        #[cfg(debug_assertions)]
        self.is_okay().unwrap();
        #[cfg(not(debug_assertions))]
        assert!(self.is_ok_quick());
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
    /// let mut board = Board::start_pos();
    /// let success = board.apply_uci_move("e2e4");
    ///
    /// assert!(success);
    /// ```
    pub fn apply_uci_move(&mut self, uci_move: &str) -> bool {
        let all_moves: MoveList = self.generate_moves();
        let bit_move: Option<BitMove> = all_moves
            .iter()
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
    /// let mut chessboard = Board::start_pos();
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

        self.turn = !self.turn;
        let us: Player = self.turn;
        let from: SQ = undo_move.get_src();
        let to: SQ = undo_move.get_dest();
        let mut piece_on: Piece = self.piece_at_sq(to);

        // Make sure the piece moved from is not there, or there is a castle
        assert!(self.piece_at_sq(from) == Piece::None || undo_move.is_castle());

        if undo_move.is_promo() {
            assert_eq!(piece_on.type_of(), undo_move.promo_piece());

            self.remove_piece_c(piece_on, to);
            self.put_piece_c(Piece::make_lossy(us, PieceType::P), to);
            piece_on = Piece::make_lossy(us, PieceType::P);
        }

        if undo_move.is_castle() {
            self.remove_castling(us, from, to);
        } else {
            self.move_piece_c(piece_on, to, from);
            let cap_piece = self.state.captured_piece;
            if !cap_piece.is_none() {
                let mut cap_sq: SQ = to;
                if undo_move.is_en_passant() {
                    match us {
                        Player::White => cap_sq -= SQ(8),
                        Player::Black => cap_sq += SQ(8),
                    };
                }
                self.put_piece_c(Piece::make_lossy(!us, cap_piece), cap_sq);
            }
        }
        self.state = self.state.get_prev().unwrap();
        self.half_moves -= 1;
        self.depth -= 1;
        self.zobrist_history.pop();

        #[cfg(debug_assertions)]
        self.is_okay().unwrap();
        #[cfg(not(debug_assertions))]
        assert!(self.is_ok_quick());
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
    /// let mut chessboard = Board::start_pos();
    /// let board_clone = chessboard.shallow_clone();
    ///
    /// unsafe { chessboard.apply_null_move(); }
    ///
    /// assert_ne!(chessboard.depth(), board_clone.depth());
    /// ```
    pub unsafe fn apply_null_move(&mut self) {
        assert!(self.checkers().is_empty());

        let mut zob: u64 = self.state.zobrist ^ z_side();

        self.depth += 1;
        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = UniqueArc::new(self.state.partial_clone());

        {
            let new_state: &mut BoardState = &mut next_arc_state;

            new_state.prev_move = BitMove::null();
            new_state.rule_50 += 1;
            new_state.ply += 1;

            new_state.prev = Some(Arc::clone(&self.state));

            if self.state.ep_square != NO_SQ {
                zob ^= z_ep(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            new_state.zobrist = zob;
            self.turn = self.turn.other_player();

            // Set the checking information
            new_state.set_check_info(self);
        }
        self.state = next_arc_state.shareable();

        #[cfg(debug_assertions)]
        self.is_okay().unwrap();
        #[cfg(not(debug_assertions))]
        assert!(self.is_ok_quick());
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
    /// let mut chessboard = Board::start_pos();
    /// let board_clone = chessboard.shallow_clone();
    ///
    /// unsafe { chessboard.apply_null_move(); }
    ///
    /// assert_ne!(chessboard.ply(), board_clone.ply());
    ///
    /// unsafe { chessboard.undo_null_move(); }
    ///
    /// assert_eq!(chessboard.moves_played(), board_clone.moves_played());
    /// assert_eq!(chessboard.fen(), board_clone.fen());
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
    /// let chessboard = Board::start_pos();
    /// let moves = chessboard.generate_moves();
    ///
    /// println!("There are {} possible legal moves.", moves.len());
    /// ```
    pub fn generate_moves(&self) -> MoveList {
        MoveGen::generate::<Legal, AllGenType>(self)
    }

    /// Get a List of legal `BitMove`s (alongside a score) for the player whose turn it is to move.
    ///
    /// This method already takes into account if the Board is currently in check, and will return
    /// legal moves only. The `ScoringMoveList` that is returned will have a value of zero for each
    /// move.
    pub fn generate_scoring_moves(&self) -> ScoringMoveList {
        MoveGen::generate_scoring::<Legal, AllGenType>(self)
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
    /// let chessboard = Board::start_pos();
    /// let capturing_moves = chessboard.generate_moves_of_type(GenTypes::Captures);
    ///
    /// assert_eq!(capturing_moves.len(), 0); // no possible captures for the starting position
    /// ```
    pub fn generate_moves_of_type(&self, gen_type: GenTypes) -> MoveList {
        match gen_type {
            GenTypes::All => MoveGen::generate::<Legal, AllGenType>(self),
            GenTypes::Captures => MoveGen::generate::<Legal, CapturesGenType>(self),
            GenTypes::Quiets => MoveGen::generate::<Legal, QuietsGenType>(self),
            GenTypes::QuietChecks => MoveGen::generate::<Legal, QuietChecksGenType>(self),
            GenTypes::Evasions => MoveGen::generate::<Legal, EvasionsGenType>(self),
            GenTypes::NonEvasions => MoveGen::generate::<Legal, NonEvasionsGenType>(self),
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
            GenTypes::All => MoveGen::generate::<PseudoLegal, AllGenType>(self),
            GenTypes::Captures => MoveGen::generate::<PseudoLegal, CapturesGenType>(self),
            GenTypes::Quiets => MoveGen::generate::<PseudoLegal, QuietsGenType>(self),
            GenTypes::QuietChecks => MoveGen::generate::<PseudoLegal, QuietChecksGenType>(self),
            GenTypes::Evasions => MoveGen::generate::<PseudoLegal, EvasionsGenType>(self),
            GenTypes::NonEvasions => MoveGen::generate::<PseudoLegal, NonEvasionsGenType>(self),
        }
    }

    //  ------- PRIVATE MUTATING FUNCTIONS -------

    /// Removes a Piece from the Board, if the color is unknown.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn remove_piece(&mut self, piece: PieceType, square: SQ) {
        let player = self.piece_locations.piece_at(square).player_lossy();

        self.remove_piece_c(Piece::make_lossy(player, piece), square);
    }

    /// Moves a Piece on the Board (if the color is unknown) from square 'from'
    /// to square 'to'.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn move_piece(&mut self, piece: PieceType, from: SQ, to: SQ) {
        let player = self.piece_locations.piece_at(from).player_lossy();
        self.move_piece_c(Piece::make_lossy(player, piece), from, to);
    }

    /// Places a Piece on the board at a given square and player.
    ///
    /// # Safety
    ///
    /// Assumes there is not already a piece at that square. If there already is,
    /// Undefined Behavior will result.
    fn put_piece_c(&mut self, piece: Piece, square: SQ) {
        let bb = square.to_bb();
        let (player, piece_ty) = piece.player_piece_lossy();
        self.bbs[PieceType::All as usize] |= bb;
        self.bbs[piece_ty as usize] |= bb;
        self.bbs_player[player as usize] |= bb;
        self.piece_locations.place(square, player, piece_ty);
        self.piece_counts[player as usize][piece_ty as usize] += 1;
        // Note: Should We set captured Piece?
    }

    /// Removes a Piece from the Board for a given player.
    ///
    /// # Panics
    ///
    /// Panics if there is a piece at the given square.
    fn remove_piece_c(&mut self, piece: Piece, square: SQ) {
        debug_assert_eq!(self.piece_at_sq(square), piece);
        let (player, piece_ty) = piece.player_piece_lossy();
        let bb = square.to_bb();
        self.bbs[PieceType::All as usize] ^= bb;
        self.bbs_player[player as usize] ^= bb;
        self.bbs[piece_ty as usize] ^= bb;

        self.piece_locations.remove(square);
        self.piece_counts[player as usize][piece_ty as usize] -= 1;
    }

    /// Moves a Piece on the Board of a given player from square 'from'
    /// to square 'to'.
    ///
    /// # Panics
    ///
    /// Panics if the two and from square are equal
    fn move_piece_c(&mut self, piece: Piece, from: SQ, to: SQ) {
        assert_ne!(from, to);
        let comb_bb: BitBoard = from.to_bb() | to.to_bb();
        let (player, piece_ty) = piece.player_piece_lossy();
        self.bbs[PieceType::All as usize] ^= comb_bb;
        self.bbs_player[player as usize] ^= comb_bb;
        self.bbs[piece_ty as usize] ^= comb_bb;

        self.piece_locations.remove(from);
        self.piece_locations.place(to, player, piece_ty);
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
        k_src: SQ,          // from, king startng spot
        to_r_orig: &mut SQ, // originally
        r_src: &mut SQ,
        r_dst: &mut SQ,
    ) {
        let king_side: bool = k_src < *to_r_orig;

        *r_src = *to_r_orig;
        if king_side {
            *to_r_orig = player.relative_square(SQ(6));
            *r_dst = player.relative_square(SQ(5));
        } else {
            *to_r_orig = player.relative_square(SQ(2));
            *r_dst = player.relative_square(SQ(3));
        }
        self.move_piece_c(Piece::make_lossy(player, PieceType::K), k_src, *to_r_orig);
        self.move_piece_c(Piece::make_lossy(player, PieceType::R), *r_src, *r_dst);
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

        self.move_piece_c(Piece::make_lossy(player, PieceType::K), k_dst, k_src);
        self.move_piece_c(Piece::make_lossy(player, PieceType::R), r_dst, r_src);
    }

    /// Outputs the Blockers of a given square.
    pub fn slider_blockers(&self, sliders: BitBoard, s: SQ, pinners: &mut BitBoard) -> BitBoard {
        let mut result: BitBoard = BitBoard(0);
        *pinners = BitBoard(0);
        let occupied: BitBoard = self.occupied();

        let mut snipers: BitBoard = sliders
            & ((rook_moves(BitBoard(0), s)
                & (self.piece_two_bb_both_players(PieceType::R, PieceType::Q)))
                | (bishop_moves(BitBoard(0), s)
                    & (self.piece_two_bb_both_players(PieceType::B, PieceType::Q))));

        while let Some(sniper_sq) = snipers.pop_some_lsb() {
            let b: BitBoard = between_bb(s, sniper_sq) & occupied;
            if b.is_not_empty() && !b.more_than_one() {
                result |= b;
                let player_at = self.piece_locations.piece_at(s).player_lossy();
                let other_occ = self.get_occupied_player(player_at);
                if (b & other_occ).is_not_empty() {
                    *pinners |= sniper_sq.to_bb();
                }
            }
        }

        result
    }

    /// Get the Player whose turn it is to move.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.turn(), Player::White);
    /// ```
    #[inline(always)]
    pub fn turn(&self) -> Player {
        self.turn
    }

    /// Return the Zobrist Hash of the board.
    #[inline(always)]
    pub fn zobrist(&self) -> u64 {
        self.state.zobrist
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
    /// let mut chessboard = Board::start_pos();
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
    pub fn piece_captured_last_turn(&self) -> PieceType {
        self.state.captured_piece
    }

    /// Get the current ply of the board.
    ///
    /// A ply is determined as the number of moves that have been played on the
    /// current `Board`. A simpler way to explain it would be counting the number
    /// of times `Board::undo_move()` can be legally applied.
    #[inline(always)]
    pub fn ply(&self) -> u16 {
        self.state.ply
    }

    /// Returns the current positional Score of the board. Positive scores are in favor
    /// of the white player, while negative scores are in favor of the black player.
    pub fn psq(&self) -> Score {
        self.state.psq
    }

    /// Get the current square of en-passant. This is defined not as the pawn that could be
    /// captured from an en-passant move, but rather the square directly behind it.
    ///
    /// # Safety
    ///
    /// While it returns a `SQ`, this square could be `SQ::NONE`, meaning there is no actual
    /// en-passant square.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,SQ};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.ep_square(), SQ::NONE);
    /// ```
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
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.occupied().0, 0xFFFF00000000FFFF);
    /// ```
    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.bbs[PieceType::All as usize]
    }

    /// Returns a if a `SQ` is empty on the current `Board`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,SQ};
    ///
    /// let chessboard = Board::start_pos();
    /// assert!(chessboard.empty(SQ::F6));
    /// assert!(!chessboard.empty(SQ::A2));
    /// ```
    #[inline(always)]
    pub fn empty(&self, sq: SQ) -> bool {
        self.piece_locations.piece_at(sq) == Piece::None
    }

    /// Get the BitBoard of the squares occupied by the given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.get_occupied_player(Player::White).0, 0x000000000000FFFF);
    /// ```
    #[inline(always)]
    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        self.bbs_player[player as usize]
    }

    /// Returns a Bitboard consisting of only the squares occupied by the White Player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,BitBoard};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.occupied_white(), BitBoard::RANK_1 | BitBoard::RANK_2);
    /// ```
    #[inline(always)]
    pub fn occupied_white(&self) -> BitBoard {
        self.bbs_player[Player::White as usize]
    }

    /// Returns a BitBoard consisting of only the squares occupied by the Black Player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,BitBoard};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.occupied_black(), BitBoard::RANK_8 | BitBoard::RANK_7);
    /// ```
    #[inline(always)]
    pub fn occupied_black(&self) -> BitBoard {
        self.bbs_player[Player::Black as usize]
    }

    /// Returns BitBoard of a single player and that one type of piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::Board;
    /// use pleco::{Player,PieceType};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.piece_bb(Player::White,PieceType::P).0, 0x000000000000FF00);
    /// ```
    #[inline(always)]
    pub fn piece_bb(&self, player: Player, piece: PieceType) -> BitBoard {
        self.bbs[piece as usize] & self.bbs_player[player as usize]
    }

    /// Returns the BitBoard of the Queens and Rooks of a given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.sliding_piece_bb(Player::White).count_bits(), 3);
    /// ```
    #[inline(always)]
    pub fn sliding_piece_bb(&self, player: Player) -> BitBoard {
        self.piece_two_bb(PieceType::Q, PieceType::R, player)
    }
    /// Returns the BitBoard of the Queens and Bishops of a given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.diagonal_piece_bb(Player::White).count_bits(), 3);
    /// ```
    #[inline(always)]
    pub fn diagonal_piece_bb(&self, player: Player) -> BitBoard {
        self.piece_two_bb(PieceType::Q, PieceType::B, player)
    }

    /// Returns the combined BitBoard of both players for a given piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,PieceType};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.piece_bb_both_players(PieceType::P).0, 0x00FF00000000FF00);
    /// ```
    #[inline(always)]
    pub fn piece_bb_both_players(&self, piece: PieceType) -> BitBoard {
        self.bbs[piece as usize]
    }

    /// Returns the combined BitBoard of both players for two pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,PieceType,BitBoard};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.piece_two_bb_both_players(PieceType::Q,PieceType::K).count_bits(), 4);
    /// ```
    #[inline(always)]
    pub fn piece_two_bb_both_players(&self, piece: PieceType, piece2: PieceType) -> BitBoard {
        self.bbs[piece as usize] ^ self.bbs[piece2 as usize]
    }

    /// Returns the `BitBoard` containing the locations of two given types of pieces for the given
    /// player.
    #[inline(always)]
    pub fn piece_two_bb(&self, piece: PieceType, piece2: PieceType, player: Player) -> BitBoard {
        self.piece_two_bb_both_players(piece, piece2) & self.bbs_player[player as usize]
    }

    /// Get the total number of pieces of the given piece and player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,PieceType};
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.count_piece(Player::White, PieceType::P), 8);
    /// ```
    #[inline(always)]
    pub fn count_piece(&self, player: Player, piece: PieceType) -> u8 {
        self.piece_counts[player as usize][piece as usize]
    }

    /// Get the total number of pieces a given player has.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,PieceType};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.count_pieces_player(Player::White), 16);
    /// ```
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        self.bbs_player[player as usize].count_bits()
    }

    /// Get the total number of pieces on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,PieceType};
    /// use pleco::core::bit_twiddles::*;
    ///
    /// let chessboard = Board::start_pos();
    /// assert_eq!(chessboard.count_all_pieces(), 32);
    /// ```
    #[inline]
    pub fn count_all_pieces(&self) -> u8 {
        self.bbs[PieceType::All as usize].count_bits()
    }

    /// Returns the PieceType, if any, at the square.
    ///
    /// # Panics
    ///
    /// Panics if the square is not a legal square.
    #[inline]
    pub fn piece_at_sq(&self, sq: SQ) -> Piece {
        debug_assert!(sq.is_okay());
        self.piece_locations.piece_at(sq)
    }

    /// Returns the square of the King for a given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleco::{Board,Player,SQ};
    ///
    /// let board = Board::start_pos();
    /// assert_eq!(board.king_sq(Player::White), SQ::E1);
    /// ```
    #[inline(always)]
    pub fn king_sq(&self, player: Player) -> SQ {
        self.piece_bb(player, PieceType::K).to_sq()
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
    ///
    /// E.g., pieces that are pinning a piece to the opponent's king. This will return the pinned
    /// pieces of both players, pinned to the given player's king.
    #[inline(always)]
    pub fn pinning_pieces(&self, player: Player) -> BitBoard {
        self.state.pinners_king[player as usize]
    }

    /// Returns the raw castling rights bits of the board.
    #[inline(always)]
    pub fn castling_bits(&self) -> u8 {
        self.state.castling.bits()
    }

    /// Return if a player has the possibility of castling for a given CastleType.
    /// This does not ensure a castling is possible for the player, just that the player
    /// has the castling-right available.
    #[inline(always)]
    pub fn can_castle(&self, player: Player, castle_type: CastleType) -> bool {
        self.state.castling.castle_rights(player, castle_type)
    }

    /// Returns the `Castling` structure of a player, which marks whether or not
    /// a player has the rights to castle.
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
        (path & self.occupied()).is_not_empty()
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

    /// Returns if the piece (if any) that was captured last move. This method does not
    /// distinguish between not having any last move played and not having a piece last captured.
    #[inline(always)]
    pub fn piece_last_captured(&self) -> PieceType {
        self.state.captured_piece
    }

    /// Returns the material key of the board.
    #[inline(always)]
    pub fn material_key(&self) -> u64 {
        self.state.material_key
    }

    /// Returns the current non-pawn material value of a player.
    #[inline(always)]
    pub fn non_pawn_material(&self, player: Player) -> Value {
        self.state.nonpawn_material[player as usize]
    }

    /// Returns the current non-pawn material value for both players.
    #[inline(always)]
    pub fn non_pawn_material_all(&self) -> Value {
        self.state.nonpawn_material[Player::White as usize]
            + self.state.nonpawn_material[Player::Black as usize]
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

    /// Return if the threefold repetition rule has been met.
    pub fn threefold_repetition(&self) -> bool {
        self.threefold_repetition
    }

    pub fn fifty_move_rule(&self) -> bool {
        self.state.rule_50 >= 50
    }

    /// Returns if the current side to move is in stalemate.
    ///
    /// This method can be computationally expensive, do not use outside of Engines.
    pub fn stalemate(&self) -> bool {
        !self.in_check()
            && (self.fifty_move_rule()
                || self.generate_moves().is_empty()
                || self.threefold_repetition())
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
        self.state.blockers_king[(!self.turn) as usize] & self.get_occupied_player(self.turn)
    }

    /// Gets the Pinned pieces for the given player. A pinned piece is defined as a piece that
    /// if suddenly removed, the player would find itself in check.
    #[inline(always)]
    pub fn pieces_pinned(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }
    /// Returns a BitBoard of possible attacks / defends to a square with a given occupancy.
    /// Includes pieces from both players.
    pub fn attackers_to(&self, sq: SQ, occupied: BitBoard) -> BitBoard {
        (pawn_attacks_from(sq, Player::Black) & self.piece_bb(Player::White, PieceType::P))
            | (pawn_attacks_from(sq, Player::White) & self.piece_bb(Player::Black, PieceType::P))
            | (knight_moves(sq) & self.piece_bb_both_players(PieceType::N))
            | (rook_moves(occupied, sq)
                & (self.bbs[PieceType::R as usize] | self.bbs[PieceType::Q as usize]))
            | (bishop_moves(occupied, sq)
                & (self.bbs[PieceType::B as usize] | self.bbs[PieceType::Q as usize]))
            | (king_moves(sq) & self.bbs[PieceType::K as usize])
    }

    /// Given a piece, square, and player, returns all squares the piece may possibly move to.
    #[inline]
    pub fn attacks_from(&self, piece: PieceType, sq: SQ, player: Player) -> BitBoard {
        match piece {
            PieceType::K => king_moves(sq),
            PieceType::P => pawn_attacks_from(sq, player),
            PieceType::N => knight_moves(sq),
            PieceType::B => bishop_moves(self.occupied(), sq),
            PieceType::R => rook_moves(self.occupied(), sq),
            PieceType::Q => queen_moves(self.occupied(), sq),
            _ => BitBoard(0),
        }
    }

    /// Returns if a pawn on a given square is passed.
    #[inline(always)]
    pub fn pawn_passed(&self, player: Player, sq: SQ) -> bool {
        (self.piece_bb(!player, PieceType::P) & passed_pawn_mask(player, sq)).is_empty()
    }

    //  ------- Move Testing -------

    /// Tests if a given pseudo-legal move is a legal. This is mostly for checking the legality of
    /// moves that were generated in a pseudo-legal fashion. Generating moves like this is faster,
    /// but doesn't guarantee legality due to the possibility of a discovered check happening.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    pub fn legal_move(&self, m: BitMove) -> bool {
        if m.get_src() == m.get_dest() {
            return false;
        }
        let us: Player = self.turn;
        let them: Player = !us;
        let src: SQ = m.get_src();
        let src_bb: BitBoard = src.to_bb();
        let dst: SQ = m.get_dest();

        // Special en_passant case
        if m.move_type() == MoveType::EnPassant {
            let k_sq: SQ = self.king_sq(us);
            let dst_bb: BitBoard = dst.to_bb();
            let captured_sq: SQ = SQ((dst.0 as i8).wrapping_sub(us.pawn_push()) as u8);
            let occupied: BitBoard = (self.occupied() ^ src_bb ^ captured_sq.to_bb()) | dst_bb;

            return (rook_moves(occupied, k_sq) & self.sliding_piece_bb(them)).is_empty()
                && (bishop_moves(occupied, k_sq) & self.diagonal_piece_bb(them)).is_empty();
        }

        let piece = self.piece_at_sq(src);
        if piece == Piece::None {
            return false;
        }

        // If Moving the king, check if the square moved to is not being attacked
        // Castles are checked during move-generation for check, so we're good there.
        if piece.type_of() == PieceType::K {
            return m.move_type() == MoveType::Castle
                || (self.attackers_to(dst, self.occupied()) & self.get_occupied_player(them))
                    .is_empty();
        }

        // Making sure not moving a pinned piece
        (self.pinned_pieces(self.turn) & src_bb).is_empty()
            || aligned(src, dst, self.king_sq(self.turn))
    }

    /// Rakes a random move and tests whether the move is pseudo-legal. Used to validate
    /// moves from the Transposition Table.
    ///
    /// # Safety
    ///
    /// Using this method does not guarantee that a move is legal. It only guarantee's that
    /// a move may possibly legal. To guarantee a move is completely legal for the position,
    /// use `Board::pseudo_legal_move()` followed by a `Board::legal_move()`.
    pub fn pseudo_legal_move(&self, m: BitMove) -> bool {
        let us = self.turn;
        let them = !us;
        let from: SQ = m.get_src();
        let to: SQ = m.get_dest();
        let to_bb = to.to_bb();
        let query: Piece = self.piece_locations.piece_at(from);

        if query == Piece::None {
            return false;
        }

        if m.incorrect_flag() {
            return false;
        }

        // Use a slower but simpler function for uncommon cases
        if m.move_type() != MoveType::Normal {
            return self.generate_pseudolegal_moves().contains(&m);
        }

        // cannot possibly be a promotion
        if m.is_promo() {
            return false;
        }

        let (player, piece): (Player, PieceType) = query.player_piece_lossy();

        if player != us {
            return false;
        }

        if (self.get_occupied_player(us) & to_bb).is_not_empty() {
            return false;
        }

        if piece == PieceType::P {
            if to.rank() == us.relative_rank(Rank::R8) {
                return false;
            }

            if (pawn_attacks_from(from, us) & self.get_occupied_player(them)  // not a Capture
                    & to_bb).is_empty()
                && !(from.0 as i8 + us.pawn_push() == to.0 as i8
                    && self.empty(to)
                    && m.is_quiet_move()) // not a single push
                && !(from.0 as i8 + 2 * us.pawn_push() == to.0 as i8
                    && m.is_double_push().0
                    && from.rank() == us.relative_rank(Rank::R2)
                    && self.empty(to)
                    && self.empty(SQ((to.0 as i8 - us.pawn_push()) as u8)))
            // Not a double push
            {
                return false;
            }
        } else if m.is_double_push().0 || (self.attacks_from(piece, from, us) & to_bb).is_empty() {
            return false;
        }

        if self.is_capture(m) ^ m.is_capture() {
            return false;
        }

        if m.is_capture() {
            let at_sq = self.piece_at_sq(to);
            if at_sq == Piece::None || at_sq.player_lossy() == us {
                return false;
            }
        }

        if self.in_check() {
            if piece != PieceType::K {
                if self.checkers().more_than_one() {
                    return false;
                }

                // Our move must be a blocking evasion or a capture of the checking piece
                if ((between_bb(self.checkers().to_sq(), self.king_sq(us)) | self.checkers())
                    & to_bb)
                    .is_empty()
                {
                    return false;
                }
            } else if (self.attackers_to(to, self.occupied() ^ from.to_bb())
                & self.get_occupied_player(them))
            .is_not_empty()
            {
                return false;
            }
        }
        true
    }

    /// Returns if the board contains only two bishops, one for each color, and each being
    /// on different squares.
    #[inline(always)]
    pub fn opposite_bishops(&self) -> bool {
        self.piece_counts[Player::White as usize][PieceType::B as usize] == 1
            && self.piece_counts[Player::Black as usize][PieceType::B as usize] == 1
            && {
                let w_bishop =
                    self.bbs_player[Player::White as usize] & self.bbs[PieceType::B as usize];
                let b_bishop =
                    self.bbs_player[Player::Black as usize] & self.bbs[PieceType::B as usize];
                w_bishop.to_sq().opposite_colors(b_bishop.to_sq())
            }
    }

    /// Checks if a move is an advanced pawn push, meaning it passes into enemy territory.
    #[inline(always)]
    pub fn advanced_pawn_push(&self, mov: BitMove) -> bool {
        self.piece_at_sq(mov.get_src()).type_of() == PieceType::P
            && self.turn().relative_rank_of_sq(mov.get_src()) > Rank::R4
    }

    /// Returns if a move is a capture.
    ///
    /// This is similar to `BitMove::is_capture`, but instead compares the move to the `Board`s
    /// data, rather than relying on the information encoded in the move.
    #[inline(always)]
    pub fn is_capture(&self, mov: BitMove) -> bool {
        assert_ne!(mov.get_dest_u8(), mov.get_src_u8());
        (!self.empty(mov.get_dest()) && mov.move_type() != MoveType::Castle)
            || mov.move_type() == MoveType::EnPassant
    }

    /// Returns if a move is a capture.
    ///
    /// This is similar to `BitMove::is_capture` & `BitMove::is_promo`, but instead compares the
    /// move to the `Board`s data, rather than relying on the information encoded in the move.
    #[inline(always)]
    pub fn is_capture_or_promotion(&self, mov: BitMove) -> bool {
        assert_ne!(mov.get_dest_u8(), mov.get_src_u8());
        if mov.move_type() != MoveType::Normal {
            mov.move_type() != MoveType::Castle
        } else {
            !self.empty(mov.get_dest())
        }
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
        let us: Player = self.turn();
        let them: Player = !us;
        let opp_king_sq: SQ = self.king_sq(them);

        // Stupidity Checks
        assert_ne!(src, dst);
        assert_eq!(self.piece_at_sq(src).player_lossy(), self.turn);

        // Searches for direct checks from the pre-computed array
        if (self.state.check_sqs[self.piece_at_sq(src).type_of() as usize] & dst_bb).is_not_empty()
        {
            return true;
        }

        // Discovered (Indirect) checks, where a sniper piece is attacking the king
        if (self.discovered_check_candidates() & src_bb).is_not_empty()  // check if the piece is blocking a sniper
            && !aligned(src, dst, opp_king_sq)
        {
            // Make sure the dst square is not aligned
            return true;
        }

        match m.move_type() {
            MoveType::Normal => false, // Nothing to check here
            MoveType::Promotion => {
                // check if the Promo Piece attacks king
                let attacks_bb = match m.promo_piece() {
                    PieceType::N => knight_moves(dst),
                    PieceType::B => bishop_moves(self.occupied() ^ src_bb, dst),
                    PieceType::R => rook_moves(self.occupied() ^ src_bb, dst),
                    PieceType::Q => queen_moves(self.occupied() ^ src_bb, dst),
                    _ => unsafe { unreachable_unchecked() },
                };
                (attacks_bb & opp_king_sq.to_bb()).is_not_empty()
            }
            MoveType::EnPassant => {
                // Check for indirect check from the removal of the captured pawn
                let captured_sq: SQ = SQ::make(dst.file(), src.rank());
                let b: BitBoard = (self.occupied() ^ src_bb ^ captured_sq.to_bb()) | dst_bb;

                let turn_sliding_p: BitBoard = self.sliding_piece_bb(us);
                let turn_diag_p: BitBoard = self.diagonal_piece_bb(us);

                ((rook_moves(b, opp_king_sq) & turn_sliding_p)
                    | (bishop_moves(b, opp_king_sq) & turn_diag_p))
                    .is_not_empty()
            }
            MoveType::Castle => {
                // Check if the rook attacks the King now
                let k_from: SQ = src;
                let r_from: SQ = dst;

                let k_to: SQ = self.turn.relative_square({
                    if r_from > k_from {
                        SQ(6)
                    } else {
                        SQ(2)
                    }
                });
                let r_to: SQ = self.turn.relative_square({
                    if r_from > k_from {
                        SQ(5)
                    } else {
                        SQ(3)
                    }
                });

                let opp_k_bb = opp_king_sq.to_bb();
                (rook_moves(BitBoard(0), r_to) & opp_k_bb).is_not_empty()
                    && (rook_moves(
                        r_to.to_bb()
                            | k_to.to_bb()
                            | (self.occupied() ^ k_from.to_bb() ^ r_from.to_bb()),
                        r_to,
                    ) & opp_k_bb)
                        .is_not_empty()
            }
        }
    }

    /// `see_ge` stands for Static Exchange Evaluation, Greater or Equal. This teats if the
    /// Static Exchange Evaluation of a move is greater than or equal to a value.
    ///
    /// This is a recursive algorithm that works by checking the destination square of
    /// the given move, and attempting to repeatedly capture that spot for both players.
    ///
    /// If the move is invalid for the current board, `false` will be returned regardless
    /// of the threshold.
    pub fn see_ge(&self, mov: BitMove, threshold: i32) -> bool {
        if mov.move_type() != MoveType::Normal {
            return 0 >= threshold;
        }

        let from = mov.get_src();
        let to = mov.get_dest();
        let mut next_victim: PieceType;

        let piece = self.piece_at_sq(from).type_of();
        if piece != PieceType::None {
            next_victim = piece;
        } else {
            return false;
        }

        let us: Player;
        let mut stm: Player;
        let mut stm_attackers: BitBoard;

        let player_us = self.piece_at_sq(from);
        if player_us != Piece::None {
            us = player_us.player_lossy();
            stm = !us;
            if us == stm {
                return false;
            }
        } else {
            return false;
        }

        // Values of the pieces taken by us minus opponent's ones
        let mut balance: i32 = piece_value(self.piece_at_sq(to), false) - threshold;

        if balance < 0 {
            return false;
        }

        // If it is enough (like in PxQ) then return immediately. Note that
        // in case nextVictim == KING we always return here, this is ok
        // if the given move is legal.
        balance -= piecetype_value(next_victim, false);

        if balance >= 0 {
            return true;
        }

        // Find all attackers to the destination square, with the moving piece
        // removed, but possibly an X-ray attacker added behind it.
        let mut occupied: BitBoard = self.occupied() ^ to.to_bb() ^ from.to_bb();
        let mut attackers: BitBoard = self.attackers_to(to, occupied) & occupied;

        loop {
            stm_attackers = attackers & self.get_occupied_player(stm);
            // Don't allow pinned pieces to attack (except the king) as long as
            // all pinners are on their original square.
            if (self.state.pinners_king[stm as usize] & !occupied).is_empty() {
                stm_attackers &= !self.state.blockers_king[stm as usize];
            }

            // If stm has no more attackers then give up: stm loses
            if stm_attackers.is_empty() {
                break;
            }

            // Locate and remove the next least valuable attacker, and add to
            // the bitboard 'attackers' the possibly X-ray attackers behind it.
            next_victim =
                self.min_attacker::<PawnType>(to, stm_attackers, &mut occupied, &mut attackers);

            // switch side to move
            stm = !stm;

            // Negamax the balance with alpha = balance, beta = balance+1 and
            // add nextVictim's value.
            //
            //      (balance, balance+1) -> (-balance-1, -balance)
            //
            assert!(balance < 0);
            balance = -balance - 1 - piecetype_value(next_victim, false);

            // If balance is still non-negative after giving away nextVictim then we
            // win. The only thing to be careful about it is that we should revert
            // stm if we captured with the king when the opponent still has attackers.
            if balance >= 0 {
                if next_victim == PieceType::K
                    && (attackers & self.get_occupied_player(stm)).is_not_empty()
                {
                    stm = !stm;
                }
                break;
            }
            if next_victim == PieceType::K {}
            assert_ne!(next_victim, PieceType::K);
        }

        us != stm
    }

    fn min_attacker<P>(
        &self,
        to: SQ,
        stm_attackers: BitBoard,
        occupied: &mut BitBoard,
        attackers: &mut BitBoard,
    ) -> PieceType
    where
        P: PieceTrait,
    {
        let p = P::piece_type();

        let b: BitBoard = stm_attackers & self.piece_bb_both_players(p);
        if b.is_empty() {
            let np = match p {
                PieceType::P => {
                    self.min_attacker::<KnightType>(to, stm_attackers, occupied, attackers)
                }
                PieceType::N => {
                    self.min_attacker::<BishopType>(to, stm_attackers, occupied, attackers)
                }
                PieceType::B => {
                    self.min_attacker::<RookType>(to, stm_attackers, occupied, attackers)
                }
                PieceType::R => {
                    self.min_attacker::<QueenType>(to, stm_attackers, occupied, attackers)
                }
                _ => self.min_attacker::<KingType>(to, stm_attackers, occupied, attackers),
            };
            return np;
        }

        *occupied ^= b.lsb();

        if p == PieceType::P || p == PieceType::B || p == PieceType::Q {
            *attackers |= bishop_moves(*occupied, to)
                & (self.piece_bb_both_players(PieceType::B)
                    | (self.piece_bb_both_players(PieceType::Q)));
        }

        if p == PieceType::R || p == PieceType::Q {
            *attackers |= rook_moves(*occupied, to)
                & (self.piece_bb_both_players(PieceType::R)
                    | (self.piece_bb_both_players(PieceType::Q)));
        }

        *attackers &= *occupied;

        p
    }

    /// Returns the piece that was moved from a given BitMove.
    ///
    /// Simply put, this method will return the `Piece` at a move's from square.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    #[inline(always)]
    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
        self.piece_at_sq(src)
    }

    /// Returns the piece that was captured, if any from a given BitMove.
    ///
    /// If the move is not a capture, `PieceType::None` will be returned.
    ///
    /// # Safety
    ///
    /// Assumes the move is legal for the current board.
    #[inline(always)]
    pub fn captured_piece(&self, m: BitMove) -> PieceType {
        if m.is_en_passant() {
            return PieceType::P;
        }
        let dst = m.get_dest();
        self.piece_at_sq(dst).type_of()
    }

    /// Returns the Zobrist key after a move is played. Doesn't recognize special
    /// moves like castling, en-passant, and promotion.
    ///
    /// # Safety
    ///
    /// Panics if the move is not legal for the current board.
    pub fn key_after(&self, m: BitMove) -> u64 {
        let src = m.get_src();
        let dst = m.get_dest();
        let piece_moved = self.piece_locations.piece_at(src);
        let piece_captured = self.piece_locations.piece_at(dst);

        let mut key: u64 = self.zobrist() ^ z_side();

        if piece_captured != Piece::None {
            key ^= z_square(dst, piece_captured);
        }

        key ^ z_square(src, piece_moved) ^ z_square(dst, piece_moved)
    }

    /// Returns a prettified String of the current `Board`, for easy command line displaying.
    ///
    /// Capital Letters represent white pieces, while lower case represents black pieces.
    pub fn pretty_string(&self) -> String {
        let mut s = String::with_capacity(SQ_CNT * 2 + 8);
        for sq in SQ_DISPLAY_ORDER.iter() {
            let op = self.piece_locations.piece_at(SQ(*sq));
            let char = if op != Piece::None {
                op.character_lossy()
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
        println!("{}", self.state.check_sqs[PieceType::B as usize]);

        println!("Rook check sqs");
        println!("{}", self.state.check_sqs[PieceType::R as usize]);

        println!("Queen check sqs");
        println!("{}", self.state.check_sqs[PieceType::Q as usize]);
    }

    /// Prints a prettified representation of the board.
    pub fn pretty_print(&self) {
        println!("{}", self.pretty_string());
    }

    /// Print the board alongside useful information.
    ///
    /// Mostly for Debugging usage.
    pub fn fancy_print(&self) {
        self.pretty_print();
        println!(
            "Castling bits: {:b}, Rule 50: {}, ep_sq: {}",
            self.state.castling, self.state.rule_50, self.state.ep_square
        );
        println!(
            "Total Moves: {}, ply: {}, depth: {}",
            self.half_moves, self.state.ply, self.depth
        );
        println!("Zobrist: {:x}", self.state.zobrist);
        println!();
    }
}

// TODO: Error Propagation

/// Errors concerning the current `Board` position.
pub enum BoardError {
    IncorrectKingNum { player: Player, num: u8 },
    IncorrectKingSQ { player: Player, sq: SQ },
    BadEPSquare { sq: SQ },
}

impl fmt::Debug for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BoardError::IncorrectKingNum { player, num } => {
                writeln!(f, "incorrect number of kings for {}: {}", player, num)
            }
            BoardError::IncorrectKingSQ { player, sq } => writeln!(
                f,
                "The board.king_sq for {} player was not at the correct location: {}",
                player, sq
            ),
            BoardError::BadEPSquare { sq } => writeln!(f, "Bad En-passant Square: {}", sq),
        }
    }
}

impl Board {
    /// Checks the basic status of the board, returning false if something is wrong.
    pub fn is_ok_quick(&self) -> bool {
        self.piece_at_sq(self.king_sq(Player::White)).type_of() == PieceType::K
            && self.piece_at_sq(self.king_sq(Player::Black)).type_of() == PieceType::K
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
        let w_king_num = self.count_piece(Player::White, PieceType::K);
        let b_king_num = self.count_piece(Player::Black, PieceType::K);
        if w_king_num != 1 {
            return Err(BoardError::IncorrectKingNum {
                player: Player::White,
                num: w_king_num,
            });
        }
        if w_king_num != 1 {
            return Err(BoardError::IncorrectKingNum {
                player: Player::Black,
                num: b_king_num,
            });
        }
        let w_ksq = self.king_sq(Player::White);
        let b_ksq = self.king_sq(Player::Black);
        if self.piece_at_sq(w_ksq).type_of() != PieceType::K {
            return Err(BoardError::IncorrectKingSQ {
                player: Player::White,
                sq: w_ksq,
            });
        }
        if self.piece_at_sq(b_ksq).type_of() != PieceType::K {
            return Err(BoardError::IncorrectKingSQ {
                player: Player::Black,
                sq: b_ksq,
            });
        }

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
    All,
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
    seed: u64,
    only_startpos: bool,
}

impl Default for RandBoard {
    fn default() -> Self {
        RandBoard {
            gen_type: RandGen::All,
            minimum_move: 2,
            favorable_player: Player::Black,
            prng: PRNG::init(1),
            seed: 0,
            only_startpos: false,
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
            seed: 0,
            only_startpos: false,
        }
    }

    /// Creates a `Vec<Board>` full of `Boards` containing random positions. The
    /// `Vec` will be of size 'size'.
    pub fn many(mut self, size: usize) -> Vec<Board> {
        let mut boards: Vec<Board> = Vec::with_capacity(size);
        for _x in 0..size {
            boards.push(self.go());
        }
        boards
    }

    /// Creates a singular `Board` with a random position.
    pub fn one(mut self) -> Board {
        self.go()
    }

    /// Turns PseudoRandom generation on. This allows for the same random `Board`s
    /// to be created from the same seed.
    pub fn pseudo_random(mut self, seed: u64) -> Self {
        self.seed = if seed == 0 { 1 } else { seed };
        self.prng = PRNG::init(seed);
        self
    }

    /// Sets the minimum moves a randomly generated `Board` must contain.
    pub fn min_moves(mut self, moves: u16) -> Self {
        self.minimum_move = moves;
        self
    }

    /// Guarantees that the boards returned are only in check,
    pub fn in_check(mut self) -> Self {
        self.gen_type = RandGen::InCheck;
        self
    }

    /// Guarantees that the boards returned are not in check.
    pub fn no_check(mut self) -> Self {
        self.gen_type = RandGen::NoCheck;
        self
    }

    /// Generates Random Boards from the start position only
    pub fn from_start_pos(mut self) -> Self {
        self.only_startpos = true;
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
            let mut board = self.select_board();
            let mut iterations = 0;
            let mut moves = board.generate_moves();

            while iterations < 100 && !moves.is_empty() {
                let mut rand = self.random() % max(90 - min(max(iterations, 0), 90), 13);
                if iterations > 20 {
                    rand %= 60;
                    if iterations > 36 {
                        rand >>= 1;
                    }
                }

                if rand == 0 && self.to_ret(&board) {
                    return board;
                }

                self.apply_random_move(&mut board);
                moves = board.generate_moves();
                iterations += 1;
            }
        }
    }

    fn select_board(&mut self) -> Board {
        if self.only_startpos || self.random() % 3 == 0 {
            Board::default()
        } else {
            let rn = self.random() % fen::ALL_FENS.len();
            Board::from_fen(fen::ALL_FENS[rn]).unwrap()
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
        let gen: bool = match self.gen_type {
            RandGen::All => true,
            RandGen::InCheck => board.in_check(),
            RandGen::NoCheck => !board.in_check(),
        };
        gen && (board.moves_played() >= self.minimum_move)
    }

    fn apply_random_move(&mut self, board: &mut Board) {
        let (rand_num, favorable): (usize, bool) = if self.favorable(board.turn) {
            (24, true)
        } else {
            (13, false)
        };

        let best_move = if self.random() % rand_num == 0 {
            let moves = board.generate_moves();
            moves[self.random() % moves.len()]
        } else if self.random() % 5 == 0 {
            AlphaBetaSearcher::best_move(board.shallow_clone(), 2)
        } else if self.random() % 3 == 0 || !favorable && self.random() % 5 < 4 {
            AlphaBetaSearcher::best_move(board.shallow_clone(), 3)
        } else {
            AlphaBetaSearcher::best_move(board.shallow_clone(), 4)
        };
        board.apply_move(best_move);
    }

    fn favorable(&self, player: Player) -> bool {
        self.gen_type == RandGen::InCheck && self.favorable_player == player
    }
}

#[cfg(test)]
mod tests {

    extern crate rand;
    use board::Board;
    use {BitMove, PieceType, SQ};

    #[test]
    fn random_move_apply() {
        let mut board = Board::start_pos();
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
        let mut board = Board::start_pos();
        let mut ply = 1000;
        let mut fen_stack = Vec::new();
        while ply > 0 && !board.checkmate() && !board.stalemate() {
            fen_stack.push(board.fen());
            let moves = board.generate_moves();
            let picked_move = moves[rand::random::<usize>() % moves.len()];
            board.apply_move(picked_move);
            ply -= 1;
        }

        while !fen_stack.is_empty() {
            board.undo_move();
            assert_eq!(board.fen(), fen_stack.pop().unwrap());
        }
    }

    #[test]
    fn zob_equality() {
        let mut board = Board::start_pos();
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
            assert_eq!(board.zobrist(), zobrist_stack.pop().unwrap());
        }
    }

    #[test]
    fn rand_board_gen_one() {
        let boards_1 = Board::random().pseudo_random(550087423).min_moves(3).one();

        let boards_2 = Board::random().pseudo_random(550087423).min_moves(3).one();

        assert_eq!(boards_1, boards_2);
    }

    #[test]
    fn rand_board_gen_many() {
        let mut boards_1 = Board::random().pseudo_random(222227835).many(5);

        let mut boards_2 = Board::random().pseudo_random(222227835).many(5);

        assert_eq!(boards_1.len(), boards_2.len());
        while !boards_1.is_empty() {
            assert_eq!(boards_1.pop(), boards_2.pop());
        }
    }

    #[test]
    fn uci_move() {
        let mut b = Board::start_pos();
        assert!(!b.apply_uci_move("a1a5"));
    }

    #[test]
    fn check_state() {
        let b = Board::start_pos();
        assert_eq!(b.count_all_pieces(), 32);
        assert!(!b.checkmate());
        assert!(!b.stalemate());

        let bmove: BitMove = BitMove::make_pawn_push(SQ::A2, SQ::A4);
        assert_eq!(b.moved_piece(bmove).type_of(), PieceType::P);
        assert_eq!(b.captured_piece(bmove), PieceType::None);
    }

    #[test]
    fn see_ge_all_fens() {
        for b in super::fen::ALL_FENS.iter() {
            see_ge_all_fens_inner(&Board::from_fen(b).unwrap());
        }
    }

    fn see_ge_all_fens_inner(b: &Board) {
        for m in b.generate_moves().iter() {
            b.see_ge(*m, 0);
        }
    }
}
