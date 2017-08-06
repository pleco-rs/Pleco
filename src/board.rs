#![crate_name = "Pleco"]

use templates::*;
use magic_helper::MagicHelper;
use movegen::MoveGen;
use bit_twiddles::*;
use piece_move::{BitMove,MoveType};
use std::option::*;
use std::sync::Arc;
use std::{mem,fmt,char};
use std::hash::{Hash,Hasher};
use test;



// Initialize MAGIC_HELPER as a static structure for everyone to use

lazy_static! {
    pub static ref MAGIC_HELPER: MagicHelper<'static,'static> = MagicHelper::new();
}


bitflags! {
    /// Structure to help with recognizing the various possibilities of castling
    ///
    /// For internal use by the Board only
    ///
    /// Keeps track two things for each player
    /// 1) What sides are possible to castle from
    /// 2) Has this player castled
    ///
    /// Does not garauntee that the player containing a castling bit can castle at that
    /// time. Rather marks that castling is a possibility, e.g. a Castling struct
    /// containing a bit marking WHITE_Q means that neither the White King or Queen-side
    /// rook has moved since the game started.
    pub struct Castling: u8 {
        const WHITE_K      = 0b0000_1000; // White has King-side Castling ability
        const WHITE_Q      = 0b0000_0100; // White has Queen-side Castling ability
        const BLACK_K      = 0b0000_0010; // Black has King-side Castling ability
        const BLACK_Q      = 0b0000_0001; // White has Queen-side Castling ability
        const WHITE_CASTLE = 0b0100_0000; // White has castled
        const BLACK_CASTLE = 0b0001_0000; // Black has castled
        const WHITE_ALL    = WHITE_K.bits // White can castle for both sides
                           | WHITE_Q.bits;
        const BLACK_ALL    = BLACK_K.bits // Black can castle for both sides
                           | BLACK_Q.bits;

    }
}

impl Castling {
    /// Removes all castling possibility for a single player
    pub fn remove_player_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= BLACK_ALL.bits,
            Player::Black => self.bits &= WHITE_ALL.bits,
        }
    }

    /// Removes King-Side castling possibility for a single player
    pub fn remove_king_side_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= !WHITE_K.bits,
            Player::Black => self.bits &= !BLACK_K.bits,
        }
    }

    /// Removes Queen-Side castling possibility for a single player
    pub fn remove_queen_side_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits &= !WHITE_Q.bits,
            Player::Black => self.bits &= !BLACK_Q.bits,
        }
    }

    /// Returns if a player can castle for a given side
    pub fn castle_rights(&self, player: Player, side: CastleType) -> bool {
        match player {
            Player::White => match side {
                CastleType::KingSide  => self.contains(WHITE_K),
                CastleType::QueenSide => self.contains(WHITE_Q),
            },
            Player::Black => match side {
                CastleType::KingSide  => self.contains(BLACK_K),
                CastleType::QueenSide => self.contains(BLACK_Q),
            }
        }
    }

    /// Sets the bits to represent a given player has castled
    pub fn set_castling(&mut self, player: Player) {
        match player {
            Player::White => self.bits |= WHITE_CASTLE.bits,
            Player::Black => self.bits |= BLACK_CASTLE.bits,
        }
    }

    /// Returns if a given player has castled
    pub fn has_castled(&self, player: Player) -> bool {
        match player {
            Player::White => self.contains(WHITE_CASTLE),
            Player::Black => self.contains(BLACK_CASTLE),
        }
    }

    /// Returns if both players have lost their ability to castle
    pub fn no_castling(&self) -> bool {
        !self.contains(WHITE_K) &&
        !self.contains(WHITE_Q) &&
        !self.contains(BLACK_K) &&
        !self.contains(BLACK_Q)
    }

    /// Returns a pretty String representing the castling state
    ///
    /// Used for FEN Strings, with ('K' | 'Q') representing white castling abilities,
    /// and ('k' | 'q') representing black castling abilities. If there are no bits set,
    /// returns a String containing "-".
    pub fn pretty_string(&self) -> String {
        if self.no_castling() {
            "-".to_owned()
        } else {
            let mut s = String::default();
            if self.contains(WHITE_K) {
                s.push('K');
            }
            if self.contains(WHITE_Q) {
                s.push('Q');
            }

            if self.contains(BLACK_K) {
                s.push('k');
            }

            if self.contains(BLACK_Q) {
                s.push('q');
            }
            assert!(!s.is_empty());
            s
        }
    }
}


impl fmt::Display for Castling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}



/// Struct to allow fast lookups for any square.
///
/// Provides the Stores if there is any piece at a square, and if so provides its color and piece type.
///
/// Piece Locations is a BLIND structure, Providing a function of  |sq| -> |Piece AND/OR Player|
/// The reverse cannot be done Looking up squares from a piece / player.
struct PieceLocations {
    // Pieces are represented by the following bit_patterns:
    // x000 -> Pawn (P)
    // x001 -> Knight(N)
    // x010 -> Bishop (B)
    // x011 -> Rook(R)
    // x100 -> Queen(Q)
    // x101 -> King (K)
    // x110 -> ??? Undefined??
    // x111 -> None
    // 0xxx -> White Piece
    // 1xxx -> Black Piece

    // array of u8's, with standard ordering mapping index to square
    data: [u8; 64]
}

impl Clone for PieceLocations {
    // Need to use transmute copy as [_;64] does not automatically implement Clone
    fn clone(&self) -> PieceLocations {
        unsafe { mem::transmute_copy(&*&self.data) }
    }
}


impl PieceLocations {
    /// Constructs a new Piece Locations with a defaulty of no pieces on the board
    pub fn blank() -> PieceLocations { PieceLocations {data: [0b0111; 64]}}

    /// Constructs a new Piece Locations with the memory at a default of Zeros
    ///
    /// This function is unsafe as Zeros represent Pawns, and therefore care mus be taken
    /// to iterate through every square and ensure the correct piece or lack of piece
    /// is placed
    pub unsafe fn default() -> PieceLocations { PieceLocations {data: [0; 64]}}

    /// Places a given piece for a given player at a certain square
    ///
    /// # Panics
    /// Panics if Square is of index higher than 63
    pub fn place(&mut self, square: SQ, player: Player, piece: Piece, ) {
        assert!(sq_is_okay(square));
        self.data[square as usize] = self.create_sq(player,piece);
    }

    /// Removes a Square
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn remove(&mut self, square: SQ) {
        assert!(sq_is_okay(square));
        self.data[square as usize] = 0b0111
    }

    /// Returns the Piece at a square, Or None if the square is empty.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63.
    pub fn piece_at(&self, square: SQ) -> Option<Piece> {
        assert!(sq_is_okay(square));
        let byte: u8 = self.data[square as usize] & 0b0111;
        match byte {
            0b0000 => Some(Piece::P),
            0b0001 => Some(Piece::N),
            0b0010 => Some(Piece::B),
            0b0011 => Some(Piece::R),
            0b0100 => Some(Piece::Q),
            0b0101 => Some(Piece::K),
            0b0110 => unreachable!(), // Undefined
            0b0111 => None,
            _ => unreachable!()
        }
    }

    /// Returns the Piece at a square for a given player.
    ///
    /// If there is no piece at that square, or there is a piece of another player at that square,
    /// returns None.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn piece_at_for_player(&self, square: SQ, player: Player) -> Option<Piece>{
        let op = self.player_piece_at(square);
        if op.is_some() {
            let p = op.unwrap();
            if p.0 == player {
                Some(p.1)
            } else { None }
        } else { None }
    }

    /// Returns the player (if any) is occupying a square
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn player_at(&self, square: SQ) -> Option<Player> {
        let byte: u8 = self.data[square as usize];
        if byte == 0b0111 || byte == 0b1111 {
            return None;
        }
        if byte < 8 {
            Some(Player::White)
        } else {
            Some(Player::Black)
        }
    }

    /// Returns a Tuple of (Player,Piece) of the player and associated piece at a
    /// given square. Returns None if the square is unoccupied.
    ///
    /// # Panics
    ///
    /// Panics if Square is of index higher than 63
    pub fn player_piece_at(&self, square: SQ) -> Option<(Player,Piece)> {
        let byte: u8 = self.data[square as usize];
        match byte {
            0b0000 => Some((Player::White, Piece::P)),
            0b0001 => Some((Player::White, Piece::N)),
            0b0010 => Some((Player::White, Piece::B)),
            0b0011 => Some((Player::White, Piece::R)),
            0b0100 => Some((Player::White, Piece::Q)),
            0b0101 => Some((Player::White, Piece::K)),
            0b0110 => unreachable!(), // Undefined
            0b0111 |  0b1111 => None,
            0b1000 => Some((Player::Black, Piece::P)),
            0b1001 => Some((Player::Black, Piece::N)),
            0b1010 => Some((Player::Black, Piece::B)),
            0b1011 => Some((Player::Black, Piece::R)),
            0b1100 => Some((Player::Black, Piece::Q)),
            0b1101 => Some((Player::Black, Piece::K)),
            0b1110 => unreachable!(), // Undefined
            _ => unreachable!()
        }
    }



    /// Returns the bits representation of a given piece and player
    fn create_sq(&self, player: Player, piece: Piece) -> u8 {
        let mut loc: u8 = match piece {
            Piece::P => 0b0000,
            Piece::N => 0b0001,
            Piece::B => 0b0010,
            Piece::R => 0b0011,
            Piece::Q => 0b0100,
            Piece::K => 0b0101,
        };
        if player == Player::Black {
            loc |= 0b1000;
        }
        loc
    }
}



/// Holds useful information concerning the current state of the board.
///
/// This is information that is computed upon making a move, and requires expensive computation to do so as well.
/// It is stored in the Heap by 'Board' as an Arc<BoardState>, as cloning the board can lead to multiple
/// references to the same BoardState.
///
/// Allows for easy undo-ing of moves as these keep track of their previous board state, forming a
/// Tree-like persistent Stack
#[derive(Clone)]
struct BoardState {
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
    /// Constructs a board state for the starting position.
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
            prev: None,
        }
    }

    /// Constructs a blank board state.
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
            prev: None,
        }
    }

    /// Constructs a partial clone of a BoardState.
    ///
    /// Castling, rule_50, ply, and ep_square are copied. The copied fields need to be
    /// modified accordingly, and the remaining fields need to be generated.
    pub fn partial_clone(&self) -> BoardState {
        BoardState {
            castling: self.castling ,
            rule_50: self.rule_50,
            ply: self.ply,
            ep_square: self.ep_square,
            zobrast: self.zobrast,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
            prev: self.get_prev(),
        }
    }

    /// Return the previous BoardState from one move ago.
    pub fn get_prev(&self) -> Option<Arc<BoardState>> {
        (&self).prev.as_ref().cloned()
    }
}




/// Represents a ChessBoard.
///
/// Board contains everything that needs to be known about the current state of the Game. It is used
/// by both Engines and Players / Bots alike.
///
/// Ideally, the Engine contains the original Representation of a board (owns the board), and utilizes
/// [Board::shallow_clone()] to share this representaion with Players.
///
/// # Examples
///
/// ```
/// use Pleco::board::*;
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
/// # BitBoard Representation
///
/// For the majority of the struct, the board utilizes [BitBoard]s, which is a u64 where each bit
/// represents an occupied location, and each bit index represents a certain square (as in bit 0 is
/// Square A1, bit 1 is B1, etc.). Indexes increase first horizontally by File, and then by Rank. See
/// [BitBoards article ChessWiki](https://chessprogramming.wikispaces.com/Bitboards) for more information.
///
/// The exact mapping from each square to bits is below,
/// ```
/// // 8 | 56 57 58 59 60 61 62 63
/// // 7 | 48 49 50 51 52 53 54 55
/// // 6 | 40 41 42 43 44 45 46 47
/// // 5 | 32 33 34 35 36 37 38 39
/// // 4 | 24 25 26 27 28 29 30 31
/// // 3 | 16 17 18 19 20 21 22 23
/// // 2 | 8  9  10 11 12 13 14 15
/// // 1 | 0  1  2  3  4  5  6  7
/// //   -------------------------
/// //     a  b  c  d  e  f  g  h
/// ```
pub struct Board {
    turn: Player, // Current turn
    bit_boards: [[BitBoard; PIECE_CNT]; PLAYER_CNT], // Occupancy per player per piece
    occ: [BitBoard; PLAYER_CNT],    // Occupancy per Player
    occ_all: BitBoard,              // BitBoard of all pieces
    half_moves: u16,                // Total moves played
    depth: u16,                     // Current depth since last shallow_copy
    piece_counts: [[u8; PIECE_CNT]; PLAYER_CNT], // Count of each Piece
    piece_locations: PieceLocations, // Mapping Squares to Pieces and Plauers
    
    // State of the Board, Un modifiable.
    // Arc to allow easy and quick copying of boards without copying memory
    // or recomputing BoardStates.
    state: Arc<BoardState>,

    // List of Moves that have been played so far.
    // Only gaurunteed to have the moves since last copy.
    undo_moves: Vec<BitMove>,

    // Reference to the pre-computed information
    pub magic_helper: &'static MAGIC_HELPER,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}

impl Board {

    /// Constructs a board from the starting position
    ///
    /// # Examples
    ///
    /// ```
    /// use Pleco::board::*;
    /// use Pleco::templates::Player;
    ///
    /// let mut chessboard = Board::default();
    /// assert_eq!(chessboard.count_pieces_player(Player::White),16);
    /// ```
    pub fn default() -> Board {
        let mut b = Board {
            turn: Player::White,
            bit_boards: return_start_bb(),
            occ: [START_WHITE_OCC, START_BLACK_OCC],
            occ_all: START_OCC_ALL,
            half_moves: 0,
            depth: 0,
            piece_counts: [[8, 2, 2, 2, 1, 1], [8, 2, 2, 2, 1, 1]],
            piece_locations: unsafe {PieceLocations::default() },
            state: Arc::new(BoardState::default()),
            undo_moves: Vec::with_capacity(100), // As this is the main board, might as well reserve space
            magic_helper: &MAGIC_HELPER
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
    pub fn shallow_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: copy_piece_bbs(&self.bit_boards),
            occ: copy_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: 0,
            piece_counts: self.piece_counts.clone(),
            piece_locations: self.piece_locations.clone(),
            state: self.state.clone(),
            undo_moves: Vec::with_capacity(16), // 32 Bytes taken up
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
    pub fn parallel_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: copy_piece_bbs(&self.bit_boards),
            occ: copy_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: self.depth,
            piece_counts: self.piece_counts.clone(),
            piece_locations: self.piece_locations.clone(),
            state: self.state.clone(),
            undo_moves: Vec::with_capacity(10), // 32 Bytes taken up
            magic_helper: &MAGIC_HELPER,
        }
    }

    /// Returns an exact clone of the current board.
    ///
    /// # Safety
    ///
    /// This method is unsafe as it can give the impression of owning and operating a board
    /// structure, rather than just being provided shallow clones.
    pub unsafe fn deep_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: copy_piece_bbs(&self.bit_boards),
            occ: copy_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: self.depth,
            piece_counts: self.piece_counts.clone(),
            piece_locations: self.piece_locations.clone(),
            state: self.state.clone(),
            undo_moves: self.undo_moves.clone(),
            magic_helper: &MAGIC_HELPER,
        }
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
                self.piece_counts[*player as usize][*piece as usize] = popcount64(self.piece_bb(*player,*piece));
            }
        }

        // Loop through each square and see if any bitboard contains something at that location, and set
        // the Boards' PieceLocations accordingly.
        for square in 0..SQ_CNT as u8 {
            let bb = sq_to_bb(square);
            if bb & self.get_occupied() != 0 {
                let player = if bb & self.occupied_black() == 0 { Player::White } else { Player::Black };
                let piece = if self.piece_bb(player, Piece::P) & bb != 0 { Piece::P }
                    else if self.piece_bb(player, Piece::N) & bb != 0 { Piece::N }
                        else if self.piece_bb(player, Piece::B) & bb != 0 { Piece::B }
                            else if self.piece_bb(player, Piece::R) & bb != 0 { Piece::R }
                                else if self.piece_bb(player, Piece::Q) & bb != 0 { Piece::Q }
                                    else if self.piece_bb(player, Piece::K) & bb != 0 { Piece::K }
                                        else { panic!() };
            self.piece_locations.place(square,player,piece);
            } else {
                // Remove the square just in case nothing eas found. Can't assume that the PieceLocations
                // represents that square as blank
                self.piece_locations.remove(square);
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
        for sq in 0..SQ_CNT as SQ {
            let player_piece = self.piece_locations.player_piece_at(sq);
            if player_piece.is_some() {
                let player: Player = player_piece.unwrap().0;
                let piece = player_piece.unwrap().1;
                let bb = sq_to_bb(sq);
                self.bit_boards[player as usize][piece as usize] |= bb;
                self.occ[player as usize] |= bb;
            }
        }
        self.occ_all = self.occupied_black() | self.occupied_white();
        for player in &ALL_PLAYERS {
            for piece in &ALL_PIECES {
                self.piece_counts[*player as usize][*piece as usize] = popcount64(self.piece_bb(*player,*piece));
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
    /// use Pleco::board::*;
    ///
    /// let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
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
    pub fn new_from_fen(fen: &str) -> Board {
        // Create blank PieceLocations and PieceCount array
        let mut piece_loc: PieceLocations = PieceLocations::blank();
        let mut piece_cnt: [[u8; PIECE_CNT]; PLAYER_CNT] = [[0;PIECE_CNT];PLAYER_CNT];

        // split the string by white space
        let det_split: Vec<&str> = fen.split_whitespace().collect();

        // must have 6 parts :
        // [ Piece Placement, Side to Move, Castling Ability, En Passant square, Half moves, full moves]
        assert_eq!(det_split.len(), 6);

        // Split the first part by '/' for locations
        let b_rep: Vec<&str> = det_split[0].split("/").collect();

        // 8 ranks, so 8 parts
        assert_eq!(b_rep.len(), 8);

        // Start with Piece Placement
        for (i, file) in b_rep.iter().enumerate() {
            // Index starts from A8, goes to H8, then A7, etc
            // A8 is 56 in our BitBoards so we start there
            let mut idx = (7 - i) * 8;

            for char in file.chars() {
                // must be a valid square
                assert!(idx < 64);
                // Count spaces
                let dig = char.to_digit(10);
                if dig.is_some() {
                    idx += dig.unwrap() as usize;
                } else {
                    // if no space, then there is a piece here
                    let piece = match char {
                        'p' | 'P' => Piece::P,
                        'n' | 'N' => Piece::N,
                        'b' | 'B' => Piece::B,
                        'r' | 'R' => Piece::R,
                        'q' | 'Q' => Piece::Q,
                        'k' | 'K' => Piece::K,
                         _  => panic!(),
                    };
                    let player = if char.is_lowercase() { Player::Black
                        } else { Player::White };
                    piece_loc.place(idx as u8,player,piece);
                    piece_cnt[player as usize][piece as usize] += 1;
                    idx += 1;
                }
            }
        }

        // Side to Move
        let turn: Player = match det_split[1].chars().next().unwrap() {
            'b' => Player::Black,
            'w' => Player::White,
            _ => panic!()
        };

        // Castle Bytes
        let mut castle_bytes = Castling::empty();
        for char in det_split[2].chars() {
            match char {
                'K' => castle_bytes |= WHITE_K,
                'Q' => castle_bytes |= WHITE_Q,
                'k' => castle_bytes |= BLACK_K,
                'q' => castle_bytes |= BLACK_Q,
                '-' => {},
                _   => panic!(),
            }
        }

        // EP square
        let mut ep_sq: SQ = NO_SQ;
        for (i, char) in det_split[3].chars().enumerate() {
            assert!(i < 2);
            if i == 0 {
                match char {
                    'a' => ep_sq += 0,
                    'b' => ep_sq += 1,
                    'c' => ep_sq += 2,
                    'd' => ep_sq += 3,
                    'e' => ep_sq += 4,
                    'f' => ep_sq += 5,
                    'g' => ep_sq += 6,
                    'h' => ep_sq += 7,
                    '-' => {},
                     _  => panic!()
                }
            } else {
                let digit = char.to_digit(10).unwrap() as u8;
                // must be 3 or 6
                assert!(digit == 3 || digit == 6);
                ep_sq += 8 * digit;
            }
        }

        // rule 50 counts
        let rule_50 = det_split[4].parse::<i16>().unwrap();

        // Total Moves Played
        // Moves is defined as everyime White moves, so gotta translate to total moves
        let mut total_moves = (det_split[5].parse::<u16>().unwrap() - 1) * 2;
        if turn == Player::Black {total_moves += 1};

        // Create the Board States
        let mut board_s = Arc::new(BoardState {
            castling: castle_bytes,
            rule_50: rule_50,
            ply: 0,
            ep_square: ep_sq,
            zobrast: 0,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
            prev: None,
        });

        // Create the Board
        let mut b = Board {
            turn: turn,
            bit_boards: [[0; PIECE_CNT];PLAYER_CNT],
            occ: [0,0],
            occ_all: 0,
            half_moves: total_moves,
            depth: 0,
            piece_counts: piece_cnt,
            piece_locations: piece_loc,
            state: Arc::new(BoardState::default()),
            undo_moves: Vec::with_capacity(20),
            magic_helper: &MAGIC_HELPER
        };

        // Set the BitBoards
        b.set_bitboards();
        {
            // Set Check info
            let mut state: &mut BoardState = Arc::get_mut(&mut board_s).unwrap();
            b.set_check_info(state);
        }
        b.state = board_s;
        // Set Zobrist Hash
        b.set_zob_hash();

        // TODO: Check for a valid FEN String and /or resulting board
        b
    }

    /// Creates a FEN String of the Given Board.
    ///
    /// FEN stands for Forsyth-Edwards Notation, and is a way of representing a board through a
    /// string of characters. More information can be found on the [ChessWiki](https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use Pleco::board::*;
    ///
    /// let board = Board::default();
    /// assert_eq!(board.get_fen(),"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// ```
    pub fn get_fen(&self) -> String {
        let mut s = String::default();
        let mut blanks = 0;
        for idx in 0..SQ_CNT as u8 {
            // Cause of weird fen ordering, gotta do it this way
            let sq = (idx % 8) + (8 * (7 - (idx / 8)));
            if file_of_sq(sq) == File::A && rank_of_sq(sq) != Rank::R8 {
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
                s.push(PIECE_DISPLAYS[player.unwrap() as usize][piece.unwrap() as usize]);
            }
        }
        s.push(' ');
        // current turn
        s.push( match self.turn {
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
            s.push(FILE_DISPLAYS[file_idx_of_sq(ep) as usize]);
            s.push(RANK_DISPLAYS[rank_idx_of_sq(ep) as usize]);
        }
        s.push(' ');
        s.push_str(&format!("{}",self.rule_50()));
        s.push(' ');
        s.push_str(&format!("{}",(self.half_moves / 2)+ 1));

        s
    }
}

// Public Move Gen & Mutation Functions
impl  Board  {

    /// Applies a move to the Board.
    ///
    /// # Example
    /// ```
    /// use Pleco::board::*;
    ///
    /// fn main() {
    ///     let mut chessboard = Board::default();
    ///
    ///     let moves = chessboard.generate_moves();
    ///     chessboard.apply_move(moves[0]);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The supplied BitMove must be both a valid move for that position, as well as a
    /// valid [BitMove], Otherwise, a panic will occur. Valid BitMoves can be generated with
    /// [Board::generate_moves()], which garuntees that only Legal moves will be created.
    pub fn apply_move(&mut self, bit_move: BitMove) {

        // Check for stupidity
        assert_ne!(bit_move.get_src(),bit_move.get_dest());

        // Does this move give check?
        let gives_check: bool = self.gives_check(bit_move);

        // Zobrist Hash
        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;

        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = Arc::new(self.state.partial_clone());

        {
            // Seperate Block to allow derefencing the BoardState
            // As there is garunteed only one owner of the Arc, this is allowed
            let mut new_state: &mut BoardState = Arc::get_mut(&mut next_arc_state).unwrap();

            // Set the prev state
            new_state.prev = Some(self.state.clone());

            // Increment these
            self.half_moves += 1;
            self.depth += 1;
            new_state.rule_50 += 1;
            new_state.ply += 1;


            let us = self.turn;
            let them = other_player(self.turn);
            let from: SQ = bit_move.get_src();
            let to: SQ = bit_move.get_dest();
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
                assert_eq!(captured.unwrap(),Piece::R);
                assert_eq!(piece,Piece::K);

                let mut k_to: SQ = 0;
                let mut r_to: SQ = 0;
                // yay helper methods
                self.apply_castling(us, from, to, &mut k_to, &mut r_to);
                zob ^= self.magic_helper.z_piece_at_sq(Piece::R,k_to) ^ self.magic_helper.z_piece_at_sq(Piece::R,r_to);
                new_state.captured_piece = None;
                // TODO: Set castling rights Zobrist
                new_state.castling.remove_player_castling(us);
                new_state.castling.set_castling(us);
            } else if captured.is_some() {
                let mut cap_sq: SQ = to;
                let cap_p: Piece = captured.unwrap(); // This shouldn't panic unless move is void
                if cap_p == Piece::P && bit_move.is_en_passant() {
                    assert_eq!(cap_sq, self.state.ep_square);
                    match us {
                        Player::White => cap_sq -= 8,
                        Player::Black => cap_sq += 8,
                    };
                    assert_eq!(piece, Piece::P);
                    assert_eq!(relative_rank(us,Rank::R6), rank_of_sq(to));
                    assert!(self.piece_at_sq(to).is_none());
                    assert_eq!(self.piece_at_sq(cap_sq).unwrap(),Piece::P);
                    assert_eq!(self.player_at_sq(cap_sq).unwrap(),them);
                    self.remove_piece_c(Piece::P,cap_sq,them);
                } else {
                    self.remove_piece_c(cap_p,cap_sq,them);
                }
                zob ^= self.magic_helper.z_piece_at_sq(cap_p,cap_sq);

                // Reset Rule 50
                new_state.rule_50 = 0;
                new_state.captured_piece = Some(cap_p);
            }

            // Update hash for moving piece
            zob ^= self.magic_helper.z_piece_at_sq(piece,to) ^ self.magic_helper.z_piece_at_sq(piece,from);

            if self.state.ep_square != NO_SQ {
                zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            // Update castling rights
            if !new_state.castling.is_empty() && !bit_move.is_castle() {
                if piece == Piece::K {
                    new_state.castling.remove_player_castling(us);
                } else if piece == Piece::R {
                    match us {
                        Player::White => {
                            if from == ROOK_WHITE_KSIDE_START {
                                new_state.castling.remove_king_side_castling(Player::White);
                            } else if from == ROOK_WHITE_QSIDE_START {
                                new_state.castling.remove_queen_side_castling(Player::White);
                            }
                        },
                        Player::Black => {
                            if from == ROOK_BLACK_KSIDE_START {
                                new_state.castling.remove_king_side_castling(Player::Black);
                            } else if from == ROOK_BLACK_QSIDE_START {
                                new_state.castling.remove_queen_side_castling(Player::Black);
                            }
                        }
                    }
                }
            }

            // Actually move the piece
            if !bit_move.is_castle() && !bit_move.is_promo() {
                self.move_piece_c(piece, from, to, us);
            }

            // Pawn Moves need special help :(
            if piece == Piece::P {
                if self.magic_helper.distance_of_sqs(to,from) == 2 {
                    // Double Push
                    new_state.ep_square = (to + from) / 2;
                    zob ^= self.magic_helper.z_ep_file(new_state.ep_square);
                } else if bit_move.is_promo() {
                    let promo_piece: Piece = bit_move.promo_piece();

                    self.remove_piece_c(Piece::P, from, us);
                    self.put_piece_c(promo_piece,to,us);
                    zob ^= self.magic_helper.z_piece_at_sq(promo_piece,to) ^ self.magic_helper.z_piece_at_sq(piece,from);
                }
                new_state.rule_50 = 0;
            }

            new_state.captured_piece = captured;
            new_state.zobrast = zob;

            if gives_check {
                new_state.checkers_bb = self.attackers_to(self.king_sq(them),self.get_occupied()) & self.get_occupied_player(us);
            }

            self.turn = them;
            self.undo_moves.push(bit_move);
            self.set_check_info(new_state); // Set the checking information
        }
        self.state = next_arc_state;
        assert!(self.is_okay());
    }

    /// Un-does the previously applied move, allowing the Board to return to it's most recently held state.
    ///
    /// # Panics
    ///
    /// Cannot be done if after a [Board::shallow_clone()] or [Board::parallel_clone()] has been done
    /// and no subsequent moves have been played:
    /// ```rust,should_panic
    /// use Pleco::board::*;
    ///
    ///
    /// let mut chessboard = Board::default();
    ///
    /// let moves = chessboard.generate_moves();
    /// chessboard.apply_move(moves[0]);
    ///
    /// let board_clone = chessboard.shallow_clone();
    ///
    /// chessboard.undo_move(); // works, chessboard existed before the move was played
    /// board_clone.undo_move(); // error: board_clone was created after the move was applied
    ///
    /// ```
    pub fn undo_move(&mut self) {
        assert!(!self.undo_moves.is_empty());
        assert!(self.state.prev.is_some());

        let undo_move: BitMove = self.undo_moves.pop().unwrap();

        self.turn = other_player(self.turn);
        let us: Player = self.turn;
        let from: SQ = undo_move.get_src();
        let to: SQ = undo_move.get_dest();
        let mut piece_on: Option<Piece> = self.piece_at_sq(to);

        // Make sure the piece moved from is not there, or there is a castle
        assert!(self.piece_at_sq(from).is_none() || undo_move.is_castle());

        if undo_move.is_promo() {
            assert_eq!(piece_on.unwrap(),undo_move.promo_piece());

            // Remove Promo piece and place Pawn in same square
            self.remove_piece_c(piece_on.unwrap(),to,us);
            self.put_piece_c(Piece::P,to,us);
            piece_on = Some(Piece::P);

        }

        if undo_move.is_castle() {
            self.remove_castling(us, from, to);
        } else {
            self.move_piece_c(piece_on.unwrap(),to,from,us);
            let cap_piece = self.state.captured_piece;

            if cap_piece.is_some() {
                let mut cap_sq: SQ = to;
                if undo_move.is_en_passant() {
                    match us {
                        Player::White => cap_sq -= 8,
                        Player::Black => cap_sq += 8,
                    };
                }
                self.put_piece_c(cap_piece.unwrap(),cap_sq,other_player(us));
            }
        }
        self.state = self.state.get_prev().unwrap();
        self.half_moves -= 1;
        self.depth -= 1;
        assert!(self.is_okay());
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
    pub unsafe fn apply_null_move(&mut self) {
        assert!(self.checkers() != 0);

        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;

        self.depth += 1;
        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = Arc::new(self.state.partial_clone());

        {
            let mut new_state: &mut BoardState = Arc::get_mut(&mut next_arc_state).unwrap();

            new_state.rule_50 += 1;
            new_state.ply += 1;

            new_state.prev = Some(self.state.clone());

            if self.state.ep_square != NO_SQ {
                zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            new_state.zobrast = zob;
            self.turn = other_player(self.turn);
            self.undo_moves.push(BitMove::null());
            self.set_check_info(new_state);
        }
        self.state = next_arc_state;
        assert!(self.is_okay());
    }

    /// Undo a "Null Move" to the Board, returning to the previous state.
    ///
    /// # Safety
    ///
    /// This method should only be used if it can be guaranteed that the last played move from
    /// the current state is a Null-Move. Otherwise, a panic will occur.
    pub unsafe fn undo_null_move(&mut self) {
        assert!(!self.undo_moves.is_empty());
        let null_move = self.undo_moves.pop().unwrap();
        assert!(null_move.is_null());
        self.turn = other_player(self.turn);
        self.state = self.state.get_prev().unwrap();
    }

    /// Get a List of legal [BitMove]s for the player whose turn it is to move.
    ///
    /// This method already takes into account if the Board is currently in check, and will return
    /// legal moves only.
    pub fn generate_moves(&self) -> Vec<BitMove> {
        MoveGen::generate(&self,GenTypes::All)
    }

    /// Get a List of legal [BitMove]s for the player whose turn it is to move or a certain type.
    ///
    /// This method already takes into account if the Board is currently in check, and will return
    /// legal moves only. If a non-ALL GenType is supplied, only a subset of the total moves will be given.
    ///
    /// # Panics
    ///
    /// Panics if given [GenTypes::QuietChecks] while the current board is in check
    pub fn generate_moves_of_type(&self, gen_type: GenTypes) -> Vec<BitMove> {
        MoveGen::generate(&self, gen_type)
    }
}

// Private Mutating Functions
impl  Board  {

    /// Helper method, used after a move is made, creates information concerning checking and
    /// possible checks.
    ///
    /// Specifically, sets Blockers, Pinners, and Check Squares for each piece.
    fn set_check_info(&self, board_state: &mut BoardState) {

        // Set the Pinners and Blockers
        let mut white_pinners = 0;
        {
            board_state.blockers_king[Player::White as usize]  =
            self.slider_blockers(self.occupied_black(), self.king_sq(Player::White), &mut white_pinners)
        };

        board_state.pinners_king[Player::White as usize] = white_pinners;

        let mut black_pinners = 0;
        {
            board_state.blockers_king[Player::Black as usize]  =
            self.slider_blockers(self.occupied_white(), self.king_sq(Player::Black), &mut black_pinners)
        };

        board_state.pinners_king[Player::Black as usize] = black_pinners;

        let ksq: SQ = self.king_sq(other_player(self.turn));
        let occupied = self.get_occupied();

        board_state.check_sqs[Piece::P as usize] = self.magic_helper.pawn_attacks_from(ksq,other_player(self.turn));
        board_state.check_sqs[Piece::N as usize] = self.magic_helper.knight_moves(ksq);
        board_state.check_sqs[Piece::B as usize] = self.magic_helper.bishop_moves(occupied, ksq);
        board_state.check_sqs[Piece::R as usize] = self.magic_helper.rook_moves(occupied, ksq);
        board_state.check_sqs[Piece::Q as usize] = board_state.check_sqs[Piece::B as usize]
                                                 | board_state.check_sqs[Piece::R as usize];
        board_state.check_sqs[Piece::K  as usize] = 0;
    }



    /// Removes a Piece from the Board, if the color is unknown.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn remove_piece(&mut self, piece: Piece, square: SQ) {
        let player = self.color_of_sq(square).unwrap();
        self.remove_piece_c(piece,square,player);
    }

    /// Moves a Piece on the Board (if the color is unknown) from square 'from'
    /// to square 'to'.
    ///
    /// # Panics
    ///
    /// Panics if there is not piece at the given square.
    fn move_piece(&mut self, piece: Piece, from: SQ, to: SQ) {
        let player = self.color_of_sq(from).unwrap();
        self.move_piece_c(piece,from,to,player);
    }

    /// Places a Piece on the board at a given square and player.
    ///
    /// # Safety
    ///
    /// Assumes there is not already a piece at that square. If there already is,
    /// Undefined Behavior will result.
    fn put_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        let bb = sq_to_bb(square);
        self.occ_all |= bb;
        self.occ[player as usize] |= bb;
        self.bit_boards[player as usize][piece as usize] |= bb;

        self.piece_locations.place(square,player,piece);
        self.piece_counts[player as usize][piece as usize] += 1;
        // Note: Should We set captured Piece?
    }

    /// Removes a Piece from the Board for a given player.
    ///
    /// # Panics
    ///
    /// Panics if there is a piece at the given square.
    fn remove_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        assert_eq!(self.piece_at_sq(square).unwrap(),piece);
        let bb = sq_to_bb(square);
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
        let comb_bb = sq_to_bb(from) | sq_to_bb(to);

        self.occ_all ^= comb_bb;
        self.occ[player as usize] ^= comb_bb;
        self.bit_boards[player as usize][piece as usize] ^= comb_bb;

        self.piece_locations.remove(from);
        self.piece_locations.place(to,player,piece);
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
    fn apply_castling(&mut self, player: Player, k_src: SQ, r_src: SQ, k_dst: &mut SQ, r_dst: &mut SQ) {
        let king_side: bool = k_src < r_src;

        if king_side {
            *k_dst = relative_square(player,6);

            *r_dst = relative_square(player,5);
        } else {
            *k_dst = relative_square(player,2);
            *r_dst = relative_square(player,3);
        }
        self.move_piece_c(Piece::K,k_src,*k_dst,player);
        self.move_piece_c(Piece::R,r_src,*r_dst,player);
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
            relative_square(player,5)
        } else {
            relative_square(player,3)
        };

        self.move_piece_c(Piece::K,k_dst,k_src,player);
        self.move_piece_c(Piece::R,r_dst,r_src,player);
    }

    /// Helper function to that outputs the Blockers of a given square
    fn slider_blockers(&self, sliders: BitBoard, s: SQ, pinners: &mut BitBoard) -> BitBoard {
        let mut result: BitBoard = 0;
        *pinners = 0;
        let occupied: BitBoard = self.get_occupied();

        let mut snipers: BitBoard = sliders & (
            (self.magic_helper.rook_moves(0, s)   & (self.piece_two_bb_both_players(Piece::R, Piece::Q)))
          | (self.magic_helper.bishop_moves(0, s) & (self.piece_two_bb_both_players(Piece::B, Piece::Q))));


        while snipers != 0 {
            let lsb: BitBoard = lsb(snipers);
            let sniper_sq: SQ = bb_to_sq(lsb);

            let b: BitBoard = self.magic_helper.between_bb(s,sniper_sq) & occupied;

            if !more_than_one(b) {
                result |= b;
                let other_occ = self.get_occupied_player(self.player_at_sq(s).unwrap());
                if b & other_occ != 0 {
                    *pinners |= sq_to_bb(sniper_sq);
                }
            }
            snipers &= !lsb;
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
        let mut b: BitBoard = self.get_occupied();
        while b != 0 {
            let sq: SQ = bit_scan_forward(b);
            let lsb: BitBoard = lsb(b);
            b &= !lsb;
            let piece = self.piece_at_bb_all(lsb);
            zob ^= self.magic_helper.z_piece_at_sq(piece.unwrap(),sq);
        }
        let ep = self.state.ep_square;
        if ep != 0 && ep < 64 {
            zob ^= self.magic_helper.z_ep_file(ep);
        }

        match self.turn {
            Player::Black =>  zob ^= self.magic_helper.z_side(),
            Player::White => {}
        };

        Arc::get_mut(&mut self.state).unwrap().zobrast = zob;
    }

}

// General information

impl Board {
    /// Get the PLayer whose turn it is to move.
    pub fn turn(&self) -> Player {self.turn}

    /// Return the Zobrist Hash.
    pub fn zobrist(&self) -> u64 {
        self.state.zobrast
    }

    /// Get the total number of moves played.
    pub fn moves_played(&self) -> u16 {
        self.half_moves
    }

    /// Get the current depth (half moves from a [Board::shallow_clone()].
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// Get the number of half-moves since a Pawn Push, castle, or capture.
    pub fn rule_50(&self) -> i16 {
        self.state.rule_50
    }

    /// Return the Piece, if any, that was last captured.
    pub fn piece_captured_last_turn(&self) -> Option<Piece> {
        self.state.captured_piece
    }

    /// Get a reference to the MagicHelper pre-computed BitBoards.
    pub fn magic_helper(&self) -> &'static MagicHelper {&MAGIC_HELPER}

    /// Get the current ply of the board.
    pub fn ply(&self) -> u16 {self.state.ply}

    /// Get the current square of en_passant.
    ///
    /// If the current en-passant square is none, it should return 64.
    pub fn ep_square(&self) -> SQ {self.state.ep_square}

}

// Position Representation
impl  Board  {
    /// Gets the BitBoard of all pieces.
    pub fn get_occupied(&self) -> BitBoard {
        self.occ_all
    }

    /// Get the BitBoard of the squares occupied by the given player.
    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        self.occ[player as usize]
    }

    /// Returns a Bitboard consisting of only the squares occupied by the White Player.
    pub fn occupied_white(&self) -> BitBoard {
        self.occ[Player::White as usize]
    }

    /// Returns a BitBoard consisting of only the squares occupied by the Black Player.
    pub fn occupied_black(&self) -> BitBoard {
        self.occ[Player::Black as usize]
    }

    /// Returns BitBoard of a single player and that one type of piece.
    pub fn piece_bb(&self, player: Player, piece: Piece) -> BitBoard {
        self.bit_boards[player as usize][piece as usize]
    }

    /// Returns the BitBoard of the Queens and Rooks of a given player.
    pub fn sliding_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::R as usize] ^ self.bit_boards[player as usize][Piece::Q as usize]
    }
    /// Returns the BitBoard of the Queens and Bishops of a given player.
    pub fn diagonal_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::B as usize] ^ self.bit_boards[player as usize][Piece::Q as usize]
    }

    /// Returns the combined BitBoard of both players for a given piece.
    pub fn piece_bb_both_players(&self, piece: Piece) -> BitBoard {
        self.bit_boards[Player::White as usize][piece as usize] ^ self.bit_boards[Player::Black as usize][piece as usize]
    }

    /// Returns the combined BitBoard of both players for two pieces.
    pub fn piece_two_bb_both_players(&self, piece: Piece, piece2: Piece) -> BitBoard {
        self.piece_bb_both_players(piece) | self.piece_bb_both_players(piece2)
    }

    /// Get the total number of pieces of the given piece and player.
    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        self.piece_counts[player as usize][piece as usize]
    }

    /// Get the total number of piees a given player has.
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        self.piece_counts[player as usize].iter().sum()
    }

    /// Get the total number of pieces on the board.
    pub fn count_all_pieces(&self) -> u8 {
        self.count_pieces_player(Player::White) + self.count_pieces_player(Player::Black)
    }

    /// Returns the piece (if any) at the given BitBoard for a given player.
    ///
    /// # Safety
    ///
    /// Number of bits must be equal to 1, or else a panic will occur.
    pub fn piece_at_bb(&self, src_bit: BitBoard, player: Player) -> Option<Piece> {
        let sq: SQ = bb_to_sq(src_bit);
        assert!(sq_is_okay(sq));
        self.piece_locations.piece_at_for_player(sq,player)
    }

    /// Returns the piece (if any) at the given BitBoard for either player.
    ///
    /// # Safety
    ///
    /// Number of bits must be equal to 1, or else a panic will occur.
    pub fn piece_at_bb_all(&self, src_bit: BitBoard)-> Option<Piece> {
        let square: SQ = bb_to_sq(src_bit);
        assert!(sq_is_okay(square));
        self.piece_locations.piece_at(square)
    }

    /// Returns the Piece, if any, at the square.
    pub fn piece_at_sq(&self, sq: SQ)-> Option<Piece> {
        assert!(sq < 64);
        self.piece_locations.piece_at(sq)
    }

    /// Returns the Player, if any, occupying the square.
    pub fn color_of_sq(&self, sq: SQ) -> Option<Player> {
        assert!(sq < 64);
        let bb: BitBoard = sq_to_bb(sq);
        if bb & self.occupied_black() != 0 { return Some(Player::Black)}
        if bb & self.occupied_white() != 0 { return Some(Player::White)}
        None
    }

    /// Returns the player, if any, at the square.
    pub fn player_at_sq(&self, s: SQ) -> Option<Player> {
        // TODO: Roll into color_of_square
        self.piece_locations.player_at(s)
    }

    /// Returns the square of the King for a given player
    pub fn king_sq(&self, player: Player) -> SQ {
        bb_to_sq(self.bit_boards[player as usize][Piece::K as usize])
    }

    /// Returns the pinned pieces of the given player.
    ///
    /// Pinned is defined as pinned to the same players king
    pub fn pinned_pieces(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }

    /// Returns the pinned pieces for a given players king. Can contain piece of from both players,
    /// but all are garunteed to be pinned to the given player's king.
    pub fn all_pinned_pieces(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize]
    }

    /// Returns the pinning pieces of a given player.
    /// e.g, pieces that are pinning a piece to the opponent's king.
    pub fn pinning_pieces(&self, player: Player) -> BitBoard {
        self.state.pinners_king[player as usize]
    }

    /// Return if a player has the possibility of castling for a given CastleType.
    pub fn can_castle(&self, player: Player, castle_type: CastleType) -> bool {
        self.state.castling.castle_rights(player,castle_type)
    }

    /// Check if the castle path is impeded for the current player.
    pub fn castle_impeded(&self, castle_type: CastleType) -> bool {
        let path: BitBoard = CASTLING_PATH[self.turn as usize][castle_type as usize];
        path & self.occ_all != 0
    }

    /// Square of the Rook that is involved with the current player's castle.
    pub fn castling_rook_square(&self, castle_type: CastleType) -> SQ {
        CASTLING_ROOK_START[self.turn as usize][castle_type as usize]
    }

    /// Return the last move played, if any.
    pub fn last_move(&self) -> Option<BitMove> {
        self.undo_moves.first().map(|b| b.clone())
    }

    /// Returns if the current player has castled ever.
    pub fn has_castled(&self, player: Player) -> bool {
        self.state.castling.has_castled(player)
    }

    /// Return if the piece (if any) that was captured last move.
    pub fn piece_last_captured(&self) -> Option<Piece> {
        self.state.captured_piece
    }
}

// Checking
impl  Board  {

    /// Return if current side to move is in check
    pub fn in_check(&self) -> bool {
        self.state.checkers_bb != 0
    }

    /// Return if the current side to move is in check_mate.
    ///
    /// This method can be computationally expensive, do not use outside of Engines.
    pub fn checkmate(&self) -> bool {
        self.in_check() && self.generate_moves().is_empty()
    }

    /// Return if the current side to move is in stalemate.
    ///
    /// This method can be computationally expensive, do not use outside of Engines.
    pub fn stalemate(&self) -> bool {
        !self.in_check() && self.generate_moves().is_empty()
    }

    /// Return the BitBoard of Checks on the current player's king.
    pub fn checkers(&self) -> BitBoard {
        self.state.checkers_bb
    }

    /// Returns the BitBoard of pieces the current side can move to discover check.
    pub fn discovered_check_candidates(&self) -> BitBoard {
        self.state.blockers_king[other_player(self.turn) as usize] & self.get_occupied_player(self.turn)
    }

    /// Gets the Pinned pieces for the given player.
    pub fn pieces_pinned(&self, player: Player) -> BitBoard {
        // TODO: combine with Board::piece_pinned
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }
    /// Returns a BitBoard of possible attacks / defends to a square with a given occupancy.
    pub fn attackers_to(&self, sq: SQ, occupied: BitBoard) -> BitBoard {
        (self.magic_helper.pawn_attacks_from(sq, Player::Black) & self.piece_bb(Player::White, Piece::P))
            | (self.magic_helper.pawn_attacks_from(sq, Player::White) & self.piece_bb(Player::Black, Piece::P))
            | (self.magic_helper.knight_moves(sq) & self.piece_bb_both_players(Piece::N))
            | (self.magic_helper.rook_moves(occupied,sq) & (self.sliding_piece_bb(Player::White) | self.sliding_piece_bb(Player::Black)))
            | (self.magic_helper.bishop_moves(occupied,sq) & (self.diagonal_piece_bb(Player::White) | self.diagonal_piece_bb(Player::Black)))
            | (self.magic_helper.king_moves(sq) & self.piece_bb_both_players(Piece::K))
    }
}


// Move Testing
impl  Board  {

    /// Tests if a given move is legal.
    pub fn legal_move(&self, m: BitMove) -> bool {

        let them: Player = other_player(self.turn);
        let src: SQ = m.get_src();
        let src_bb: BitBoard = sq_to_bb(src);
        let dst: SQ = m.get_dest();

        // Special en_passant case
        if m.move_type() == MoveType::EnPassant {
            let k_sq: SQ = self.king_sq(self.turn);
            let dst_bb: BitBoard = sq_to_bb(dst);
            let captured_sq: SQ = (dst as i8).wrapping_sub(pawn_push(self.turn)) as u8;
            let occupied: BitBoard = (self.get_occupied() ^ src_bb ^ sq_to_bb(captured_sq)) | dst_bb;

            return (self.magic_helper.rook_moves(occupied,k_sq) & self.sliding_piece_bb(them) == 0)
            && (self.magic_helper.queen_moves(occupied,k_sq) & self.diagonal_piece_bb(them) == 0)
        }

        // If Moving the king, check if the square moved to is not being attacked
        // Castles are checking during move gen for check, so we goo dthere
        if self.piece_at_sq(src).unwrap() == Piece::K {
            return m.move_type() == MoveType::Castle || (self.attackers_to(dst,self.get_occupied()) & self.get_occupied_player(them)) == 0
        }

        // Making sure not moving a pinned piece
        (self.pinned_pieces(self.turn) & src_bb) == 0 || self.magic_helper.aligned(src, dst, self.king_sq(self.turn))
    }

    // Used to check for Hashing errors from TT Tables
//    pub fn pseudo_legal_move(&self, m: BitMove) -> bool {
//        let us = self.turn;
//        let them = other_player(us);
//
//    }

    /// Returns if a move will give check to the opposing player's King.
    pub fn gives_check(&self, m: BitMove) -> bool {
        // I am too drunk to be making this right now
        let src: SQ = m.get_src();
        let dst: SQ = m.get_dest();
        let src_bb: BitBoard = sq_to_bb(src);
        let dst_bb: BitBoard = sq_to_bb(dst);
        let opp_king_sq: SQ = self.king_sq(other_player(self.turn));

        // Stupidity Checks
        assert_ne!(src, dst);
        assert_eq!(self.color_of_sq(src).unwrap(),self.turn);

        // Searches for direct checks from the pre-computed array
        if self.state.check_sqs[self.piece_at_sq(src).unwrap() as usize] & dst_bb != 0 {
            return true;
        }

        // Discovered (Indirect) checks, where a sniper piece is attacking the king
        if (self.discovered_check_candidates() & src_bb != 0)  // check if the piece is blocking a sniper
            && !self.magic_helper.aligned(src, dst, opp_king_sq) { // Make sure the dst square is not aligned
            return true;
        }

        match m.move_type() {
            MoveType::Normal => false, // Nothing to check here
            MoveType::Promotion => { // check if the Promo Piece attacks king
                let attacks_bb = match m.promo_piece() {
                    Piece::N => self.magic_helper.knight_moves(dst),
                    Piece::B => self.magic_helper.bishop_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::R => self.magic_helper.rook_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::Q => self.magic_helper.queen_moves(self.get_occupied() ^ src_bb, dst),
                    _ => unreachable!()
                };
                 attacks_bb & sq_to_bb(opp_king_sq) != 0
            },
            MoveType::EnPassant => {
                // Check for indirect check from the removal of the captured pawn
                let captured_sq: SQ = make_sq(file_of_sq(dst), rank_of_sq(src));
                let b: BitBoard = (self.get_occupied() ^ src_bb ^ sq_to_bb(captured_sq)) | dst_bb;

                let turn_sliding_p: BitBoard = self.sliding_piece_bb(self.turn);
                let turn_diag_p: BitBoard = self.diagonal_piece_bb(self.turn);

                // TODO: is this right?
                (self.magic_helper.rook_moves(b, opp_king_sq) | turn_sliding_p)
                    & (self.magic_helper.bishop_moves(b, opp_king_sq) | turn_diag_p) != 0
            },
            MoveType::Castle => {
                // Check if the rook attacks the King now
                let k_from: SQ = src;
                let r_from: SQ = dst;

                let k_to: SQ = relative_square(self.turn, {
                    if r_from > k_from { 6 } else { 2 }
                });
                let r_to: SQ = relative_square(self.turn, {
                    if r_from > k_from { 5 } else { 3 }
                });

                let opp_k_bb = sq_to_bb(opp_king_sq);
                (self.magic_helper.rook_moves(0, r_to) & opp_k_bb != 0)
                    && (self.magic_helper.rook_moves(sq_to_bb(r_to) | sq_to_bb(k_to) | (self.get_occupied() ^ sq_to_bb(k_from) ^ sq_to_bb(r_from)), r_to) & opp_k_bb) != 0
            }
        }
    }

    /// Returns the piece that was moved from a given BitMove.
    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
        self.piece_at_sq(src).unwrap() // panics if no piece here :)
    }

    /// Returns the piece that was captured, if any from a given BitMove.
    pub fn captured_piece(&self, m: BitMove) -> Option<Piece> {
        if m.is_en_passant() {
            return Some(Piece::P);
        }
        let dst = m.get_dest();
        self.piece_at_bb(sq_to_bb(dst),other_player(self.turn))
    }

}

// Printing and Debugging Functions
impl  Board  {

    /// Returns a prettified String of the current board, for Quick Display.
    ///
    /// Capital Letters represent White pieces, while lower case represents Black pieces.
    pub fn pretty_string(&self) -> String {
        let mut s = String::with_capacity(SQ_CNT * 2 + 8);
        for sq in SQ_DISPLAY_ORDER.iter() {
            let op = self.piece_locations.player_piece_at(*sq);
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

    /// Return the current ARC count of the board's BoardState
    pub fn get_arc_strong_count(&self) -> usize {
        Arc::strong_count(&self.state)
    }

    /// Get Debug Information.
    pub fn print_debug_info(&self) {
        println!("White Pinners ");
        print_bitboard(self.state.pinners_king[0]);
        println!("Black Pinners ");
        print_bitboard(self.state.pinners_king[1]);

        println!("White Blockers ");
        print_bitboard(self.state.blockers_king[0]);
        println!("Black Blockers ");
        print_bitboard(self.state.blockers_king[1]);

        println!("Checkers ");
        print_bitboard(self.state.checkers_bb);

        println!("Bishop check sqs");
        print_bitboard(self.state.check_sqs[Piece::B as usize]);

        println!("Rook check sqs");
        print_bitboard(self.state.check_sqs[Piece::R as usize]);

        println!("Queen check sqs");
        print_bitboard(self.state.check_sqs[Piece::Q as usize]);
    }

    /// Prints a prettified representation of the board.
    pub fn pretty_print(&self) {
        println!("{}",self.pretty_string());
    }

    /// Print the board alongside useful information.
    ///
    /// Mostly for Debugging useage.
    pub fn fancy_print(&self) {
        self.pretty_print();
        println!("Castling bits: {:b}, Rule 50: {}, ep_sq: {}", self.state.castling, self.state.rule_50, self.state.ep_square);
        println!("Total Moves: {}, ply: {}, depth: {}", self.half_moves, self.state.ply, self.depth);
        println!("Zobrist: {:x}", self.state.zobrast);
        println!();


    }
    // Checks the current state of the Board
    // yup
    pub fn is_okay(&self) -> bool {
        const QUICK_CHECK: bool = false;

        if QUICK_CHECK {
            return self.check_basic()
        }
        self.check_basic()
            && self.check_bitboards()
            && self.check_king()
            && self.check_state_info()
            && self.check_lists()
            && self.check_castling()
    }
}

// Debugging helper Functions
// Returns false if the board is not good
impl  Board  {
    fn check_basic(&self) -> bool {
        assert_eq!(self.piece_at_sq(self.king_sq(Player::White)).unwrap(), Piece::K);
        assert_eq!(self.piece_at_sq(self.king_sq(Player::Black)).unwrap(), Piece::K);
        assert!(self.state.ep_square == 0 || self.state.ep_square == 64 || relative_rank_of_sq(self.turn,self.state.ep_square) == Rank::R6);
        true
    }

    fn check_king(&self) -> bool {
        // TODO: Implement attacks to opposing king must be zero
        assert_eq!(self.count_piece(Player::White, Piece::K), 1);
        assert_eq!(self.count_piece(Player::Black, Piece::K), 1);
        true
    }

    fn check_bitboards(&self) -> bool {
        assert_eq!(self.occupied_white() & self.occupied_black(), 0);
        assert_eq!(self.occupied_black() | self.occupied_white(), self.get_occupied());

        // TODO: Loop through all pieces and make sure no two pieces are on the same square

        true
    }

    fn check_state_info(&self) -> bool {
        true
    }

    fn check_lists(&self) -> bool {
        true
    }

    fn check_castling(&self) -> bool {
        true
    }
}





// Testing
//
//#[test]
//fn piece_locations_cloning() {
//    let mut p = PieceLocations::default();
//    p.place(23,Player::White, Piece::Q);
//    let mut q = p.clone();
//    assert_eq!(q.piece_at(23).unwrap(),Piece::Q);
//    q.remove(23);
//    assert!(q.piece_at(23).is_none());
//    assert_eq!(p.piece_at(23).unwrap(),Piece::Q);
//}
//
//
//
//


