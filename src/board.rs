use templates::Piece as Piece;
use templates::Player as Player;
use templates::*;
use magic_helper::MagicHelper;
use bit_twiddles::*;
use piece_move::{BitMove,MoveType};
use fen;
use movegen;
use lazy_static;
use std::sync::Arc;
use std::option::*;
use std::mem;




// Initialize MAGIC_HELPER

lazy_static! {
    pub static ref MAGIC_HELPER: MagicHelper<'static,'static> = MagicHelper::new();
}

// ***** CASTLING STRUCT ***** //


// ***** BOARD STATE ***** //

// State of the Board
#[derive(Clone)]
pub struct BoardState {
    // Automatically Created
    pub castling: u8,
    pub rule_50: i8,
    pub ply: u8,
    pub ep_square: SQ,

    // Recomputed after a move
    pub zobrast: u64,
    pub captured_piece: Option<Piece>,
    pub checkers_bb: BitBoard,
    pub blockers_king: [BitBoard; PLAYER_CNT],
    pub pinners_king: [BitBoard; PLAYER_CNT],
    pub check_sqs: [BitBoard; PIECE_CNT],

    //  castling      ->  0000WWBB, left = 1 -> king side castle possible, right = 1 -> queen side castle possible
    //  rule50        -> 50 moves without capture, for draws
    //  ply           -> How many moves deep this current thread is
    //  ep_square     -> square of en_passant, if any
    //  zobrast       -> zobrast key
    //  capture_piece -> If a piece was recently captured
    //  checkers_bb   -> Bitboard of all pieces where the king is in check
    //  blockers_king -> per each player, bitboard of pieces blocking an attack on a king. Of BOTH Sides
    //  pinners_king  -> Per each player, bitboard of pieces currently pinning the opponent's king
    //  check_sqs     -> Array of pieces where check is there
}

impl BoardState {
    // Beginning Moves only
    pub fn default() -> BoardState {
        BoardState {
            castling: 0b00001111,
            rule_50: 0,
            ply: 0,
            ep_square: 0,
            zobrast: 0,
            captured_piece: None,
            checkers_bb: 0,
            blockers_king: [0; PLAYER_CNT],
            pinners_king: [0; PLAYER_CNT],
            check_sqs: [0; PIECE_CNT],
        }
    }

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
        }
    }
}

// ***** PIECE STATES ***** //


// Keeps Tracks of piece counts and the location of pieces on the board
pub struct PieceStates {
    pub piece_counts: [[u8; PIECE_CNT]; PLAYER_CNT],
    pub piece_squares: [Option<Piece>; SQ_CNT],
}

impl PieceStates {
    pub fn default() -> PieceStates {
        PieceStates {
            piece_counts: [[0; PIECE_CNT]; PLAYER_CNT],
            piece_squares: [None; SQ_CNT],
        }
    }

    pub fn clone(&self) -> PieceStates {
        let mut s = PieceStates {
            piece_counts: self.piece_counts.clone(),
            piece_squares: [None; SQ_CNT],
        };
        for i in 0..self.piece_squares.len() {
            if self.piece_squares[i].is_none() {
                s.piece_squares[i] = None;
            } else {
                s.piece_squares[i] = Some(self.piece_squares[i].unwrap().clone());
            }
        }
        s
    }
}

// ***** BOARD ***** //

pub struct Board {
    // Basic information
    pub turn: Player,
    pub bit_boards: [[BitBoard; PIECE_CNT]; PLAYER_CNT], // Occupancy per player per piece
    pub occ: [BitBoard; PLAYER_CNT], // Occupancy per Player
    pub occ_all: BitBoard, // Total Occupancy BB
    pub half_moves: u16, // Total moves
    pub depth: u8, // current depth from actual position
    pub piece_states: PieceStates, // Piece Counts and Piece Board

    // State of the Board
    pub state: BoardState,

    // Not copied
    pub undo_moves: Vec<BitMove>,
    pub move_states: Vec<BoardState>,

    // Special Case
    pub magic_helper: &'static MAGIC_HELPER,
}

//  8 | 56 57 58 59 60 61 62 63
//  7 | 48 49 50 51 52 53 54 55
//  6 | 40 41 42 43 44 45 46 47
//  5 | 32 33 34 35 36 37 38 39
//  4 | 24 25 26 27 28 29 30 31
//  3 | 16 17 18 19 20 21 22 23
//  2 | 8  9  10 11 12 13 14 15
//  1 | 0  1  2  3  4  5  6  7
//    -------------------------
//      a  b  c  d  e  f  g  h



// Initializers!
impl Board {

    // Default, starting board
    pub fn default() -> Board {
        let mut b = Board {
            turn: Player::White,
            bit_boards: return_start_bb(),
            occ: [START_WHITE_OCC, START_BLACK_OCC],
            occ_all: START_OCC_ALL,
            half_moves: 0,
            depth: 0,
            piece_states: PieceStates::default(),
            state: BoardState::default(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: &MAGIC_HELPER
        };
        b.set_zob_hash();
        b.set_piece_states();
        b
    }

    // Simple Version for testing, Skips creation of MagicHelper
    pub fn simple() -> Board {
        let mut b = Board {
            turn: Player::White,
            bit_boards: copy_piece_bbs(&START_BIT_BOARDS),
            occ: copy_occ_bbs(&START_OCC_BOARDS),
            occ_all: START_OCC_ALL,
            half_moves: 0,
            depth: 0,
            piece_states: PieceStates::default(),
            state: BoardState::default(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: &MAGIC_HELPER
        };
        b.set_piece_states();
        b
    }

    // Returns Shallow clone of current board with no Past Move List
    pub fn shallow_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: copy_piece_bbs(&self.bit_boards),
            occ: copy_occ_bbs(&self.occ),
            occ_all: self.occ_all,
            half_moves: self.half_moves,
            depth: self.depth,
            piece_states: self.piece_states.clone(),
            state: self.state.clone(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: &MAGIC_HELPER,
        }
    }

    // Sets the piece states for non cloning initilization
    fn set_piece_states(&mut self) {
        for player in ALL_PLAYERS.iter() {
            for piece in ALL_PIECES.iter() {
                self.piece_states.piece_counts[*player as usize][*piece as usize] = popcount64(self.piece_bb(*player,*piece));
            }
        }

        for square in 0..SQ_CNT as u8 {
            self.piece_states.piece_squares[square as usize] = self.piece_at_sq(square);
        }

    }

    // Creates a new Board from a fen string
    pub fn new_from_fen(fen: String) -> Result<Board, String> {
        unimplemented!();
        // https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
        // "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",
//        fen::generate_board(fen)
    }
}

// Public Move Gen & Mutation Functions
impl  Board  {

    // Applies the bitmove to the board
    pub fn apply_move(&mut self, bit_move: BitMove) {
        assert_ne!(bit_move.get_src(),bit_move.get_dest());

        let gives_check: bool = self.gives_check(bit_move);

        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;
        let mut new_state: BoardState = self.state.partial_clone();

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
            self.piece_at_sq(from)
        };

        assert_eq!(self.color_of_sq(from).unwrap(), us);

        if bit_move.is_castle() {
            assert_eq!(captured.unwrap(),Piece::R);
            assert_eq!(piece,Piece::K);

            let mut k_to: SQ = 0;
            let mut r_to: SQ = 0;
            self.apply_castling(us, to, from, &mut k_to, &mut r_to);
            zob ^= self.magic_helper.z_piece_at_sq(Piece::R,k_to) ^ self.magic_helper.z_piece_at_sq(Piece::R,r_to);
            new_state.captured_piece = None;
            new_state.castling &= !CASTLE_RIGHTS[us as usize];
        }

        // A piece has been captured
        if captured.is_some() {
            let mut cap_sq: SQ = to;
            let cap_p: Piece = captured.unwrap();
            if cap_p == Piece::P && bit_move.move_type() == MoveType::EnPassant {
                match us {
                    Player::White => cap_sq -= 8,
                    Player::Black => cap_sq += 8,
                };
                assert_eq!(piece, Piece::P);
                assert_eq!(cap_sq, self.state.ep_square);
                assert_eq!(relative_rank(us,6), rank_of_sq(to));
                assert!(self.piece_at_sq(to).is_none());
                assert_eq!(self.piece_at_sq(cap_sq).unwrap(),Piece::P);
                assert_eq!(self.player_at_sq(cap_sq).unwrap(),them);
                self.remove_piece_c(Piece::P,cap_sq,them);
            } else {
                self.remove_piece_c(cap_p,cap_sq,them);
            }
            zob ^= self.magic_helper.z_piece_at_sq(cap_p,cap_sq);
            new_state.rule_50 = 0;
        }

        zob ^= self.magic_helper.z_piece_at_sq(piece,to) ^ self.magic_helper.z_piece_at_sq(piece,from);

        if self.state.ep_square != NO_SQ {
            zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
            new_state.ep_square = NO_SQ;
        }

        if new_state.castling != 0 && !bit_move.is_castle() {
            if piece == Piece::K {
                new_state.castling &= !CASTLE_RIGHTS[us as usize];
            } else if piece == Piece::R {
                match us {
                    Player::White => {
                        if from == ROOK_WHITE_KSIDE_START {
                            new_state.castling &= !CASTLE_RIGHTS_WHITE_K;
                        } else if from == ROOK_WHITE_QSIDE_START {
                            new_state.castling &= !CASTLE_RIGHTS_WHITE_Q;
                        }
                    },
                    Player::Black => {
                        if from == ROOK_BLACK_KSIDE_START {
                            new_state.castling &= !CASTLE_RIGHTS_BLACK_K;
                        } else if from == ROOK_BLACK_QSIDE_START {
                            new_state.castling &= !CASTLE_RIGHTS_BLACK_Q;
                        }
                    }
                }
            }
        }

        if !bit_move.is_castle() && !bit_move.is_promo() {
            self.move_piece_c(piece, to, from, us);
        }

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

        if self.gives_check(bit_move) {
            new_state.checkers_bb = self.attackers_to(self.king_sq(them),self.get_occupied());
        }

        self.turn = them;
        self.move_states.push(unsafe { mem::transmute_copy(&self.state) });
        self.undo_moves.push(bit_move);
        self.state = new_state;

        self.set_check_info();
        assert!(self.is_okay());
    }

    pub fn undo_move(&mut self) {
        unimplemented!();
    }

    pub fn generate_moves(&self) -> Vec<BitMove> {
        unimplemented!();
//        movegen::get_moves(&self)
    }
}

// Private Mutating Functions
impl  Board  {

    // After a move is made, Information about the checking situation is created
    fn set_check_info(&mut self) {
        let mut white_pinners = 0;
        self.state.blockers_king[Player::White as usize]  = {
            self.slider_blockers(self.occupied_black(), self.king_sq(Player::White), &mut white_pinners) };
        self.state.pinners_king[Player::White as usize] = white_pinners;

        let mut black_pinners = 0;
        self.state.blockers_king[Player::Black as usize]  = {
            self.slider_blockers(self.occupied_white(), self.king_sq(Player::Black), &mut black_pinners) };
        self.state.pinners_king[Player::Black as usize] = black_pinners;

        let ksq: SQ = self.king_sq(other_player(self.turn));
        let occupied = self.get_occupied();

        self.state.check_sqs[Piece::P as usize] = self.magic_helper.pawn_attacks_from(ksq,other_player(self.turn));
        self.state.check_sqs[Piece::N as usize] = self.magic_helper.knight_moves(ksq);
        self.state.check_sqs[Piece::B as usize] = self.magic_helper.bishop_moves(occupied, ksq);
        self.state.check_sqs[Piece::R as usize] = self.magic_helper.rook_moves(occupied, ksq);
        self.state.check_sqs[Piece::Q as usize] = self.state.check_sqs[Piece::B as usize]
                                                 | self.state.check_sqs[Piece::R as usize];
        self.state.check_sqs[Piece::K  as usize] = 0;
    }

    // Remove a piece, color is unknown
    fn remove_piece(&mut self, piece: Piece, square: SQ) {
        let player = self.color_of_sq(square).unwrap();
        self.remove_piece_c(piece,square,player);
    }

    // move a piece, color is unknown
    fn move_piece(&mut self, piece: Piece, from: SQ, to: SQ) {
        let player = self.color_of_sq(from).unwrap();
        self.move_piece_c(piece,from,to,player);
    }

    // put a piece, color is known
    fn put_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        let bb = sq_to_bb(square);
        self.occ_all |= bb;
        self.occ[player as usize] |= bb;
        self.bit_boards[player as usize][piece as usize] |= bb;

        self.piece_states.piece_squares[square as usize] = Some(piece);
        self.piece_states.piece_counts[player as usize][piece as usize] += 1;
        // Note: Should We set captured Piece?
    }

    // remove a piece, color is known
    fn remove_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        assert_eq!(self.piece_at_sq(square).unwrap(),piece);
        let bb = sq_to_bb(square);
        self.occ_all ^= bb;
        self.occ[player as usize] ^= bb;
        self.bit_boards[player as usize][piece as usize] ^= bb;

        self.piece_states.piece_squares[square as usize] = None;
        self.piece_states.piece_counts[player as usize][piece as usize] -= 1;
    }

    // move a piece, color is known
    fn move_piece_c(&mut self, piece: Piece, from: SQ, to: SQ, player: Player) {
        assert_ne!(from, to);
        assert_eq!(self.piece_at_sq(from).unwrap(),piece);
        let comb_bb = sq_to_bb(from) | sq_to_bb(to);

        self.occ_all ^= comb_bb;
        self.occ[player as usize] ^= comb_bb;
        self.bit_boards[player as usize][piece as usize] ^= comb_bb;

        self.piece_states.piece_squares[from as usize] = None;
        self.piece_states.piece_squares[from as usize] = Some(piece);
    }

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

    fn remove_castling(&mut self, player: Player, k_src: SQ, r_src: SQ) {
        let mut k_dst: SQ = self.king_sq(player);
        let mut r_dst: SQ = 0;
        let king_side: bool = k_src < r_src;

        if king_side {
            r_dst = relative_square(player,7);
        } else {
            r_dst = relative_square(player,0);
        }
        self.move_piece_c(Piece::K,k_dst,k_src,player);
        self.move_piece_c(Piece::K,r_dst,r_src,player);
    }

    fn slider_blockers(&self, sliders: BitBoard, s: SQ, pinners: &mut BitBoard) -> BitBoard {
        let mut result: BitBoard = 0;
        *pinners = 0;
        let occupied: BitBoard = self.get_occupied();

        let mut snipers: BitBoard = sliders & (
            (self.magic_helper.rook_moves(0, s) & (self.piece_two_bb_both_players(Piece::B, Piece::Q)))
                | (self.magic_helper.bishop_moves(0, s) & (self.piece_two_bb_both_players(Piece::B, Piece::Q))));

        while snipers != 0 {
            let lsb: BitBoard = lsb(snipers);
            snipers &= !lsb;
            let sniper_sq: SQ = bb_to_sq(lsb);
            let b: BitBoard = self.magic_helper.between_bb(s,sniper_sq) & occupied;
            if !more_than_one(b) {
                result |= b;

                if b & self.get_occupied_player(self.player_at_sq(s).unwrap()) != 0 {
                    *pinners |= sq_to_bb(sniper_sq);
                }
            }
        }

        result
    }
}


// Zobrist and move making for hashing
impl  Board  {

    //    pub struct Zobrist {
    //      sq_piece: [[u64; PIECE_CNT]; SQ_CNT],
    //      en_p: [u64; FILE_CNT],
    //      castle: [u64; CASTLING_CNT],
    //      side: u64,
    //    }

    // Used to create a hash of self when initialized
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
        }

        let c = self.state.castling;
        assert!((c as usize) < CASTLING_CNT);
        zob ^= self.magic_helper.z_castle_rights(c);
        self.state.zobrast = zob;
    }



}

// Position Representation
impl  Board  {
    // Gets all occupied Squares
    pub fn get_occupied(&self) -> BitBoard { self.occ_all }

    // Get the BitBoard of the squares occupied by player
    pub fn get_occupied_player(&self, player: Player) -> BitBoard { self.occ[player as usize] }

    // Returns a Bitboard consisting of only the squares occupied by the White Player
    pub fn occupied_white(&self) -> BitBoard { self.occ[Player::White as usize] }

    // Returns a BitBoard consisting of only the squares occupied by the Black Player
    pub fn occupied_black(&self) -> BitBoard { self.occ[Player::Black as usize] }

    // Returns Bitboard for one Piece and One Player
    pub fn piece_bb(&self, player: Player, piece: Piece) -> BitBoard {
        self.bit_boards[player as usize][piece as usize]
    }

    // Horizontally moving and Vertically moving pieces of player (Queens and Rooks)
    pub fn sliding_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::R as usize] ^ self.bit_boards[player as usize][Piece::Q as usize]
    }
    // reutns BitBoard of Diagonal moving pieces (Queens and Bishops)
    pub fn diagonal_piece_bb(&self, player: Player) -> BitBoard {
        self.bit_boards[player as usize][Piece::B as usize] ^ self.bit_boards[player as usize][Piece::Q as usize]
    }

    // Bitboard of the pieces of both sides
    pub fn piece_bb_both_players(&self, piece: Piece) -> BitBoard {
        self.bit_boards[Player::White as usize][piece as usize] ^ self.bit_boards[Player::White as usize][piece as usize]
    }

    // BitBoard of both players for both pieces
    pub fn piece_two_bb_both_players(&self, piece: Piece, piece2: Piece) -> BitBoard {
        self.piece_bb_both_players(piece) | self.piece_bb_both_players(piece2)
    }

    // Total number of pieces of type Piece and of player P
    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        self.piece_states.piece_counts[player as usize][piece as usize]
    }

    // Total number of pieces of Player
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        self.piece_states.piece_counts[player as usize].iter().sum()
    }

    // Returns the piece at the given place. Number of bits must be equal to 1, or else won't work
    pub fn piece_at_bb(&self, src_bit: BitBoard, player: Player) -> Option<Piece> {
        if self.piece_bb(player, Piece::P) & src_bit != 0 { return Some(Piece::P) };
        if self.piece_bb(player, Piece::R) & src_bit != 0 { return Some(Piece::R) };
        if self.piece_bb(player, Piece::N) & src_bit != 0 { return Some(Piece::N) };
        if self.piece_bb(player, Piece::Q) & src_bit != 0 { return Some(Piece::Q) };
        if self.piece_bb(player, Piece::B) & src_bit != 0 { return Some(Piece::B) };
        if self.piece_bb(player, Piece::K) & src_bit != 0 { return Some(Piece::K) };
        None
    }

    // Returns the piece at the given place. Number of bits must be equal to 1, or else won't work
    pub fn piece_at_bb_all(&self, src_bit: BitBoard)-> Option<Piece> {
        let square: SQ = bb_to_sq(src_bit);
        assert!(sq_is_okay(square));
        self.piece_states.piece_squares[square as usize]
    }

    // Returns the Piece, if any, at the square
    pub fn piece_at_sq(&self, sq: SQ)-> Option<Piece> {
        assert!(sq < 64);
        self.piece_states.piece_squares[sq as usize]
    }

    // Returns the Player, if any, occupying the square
    pub fn color_of_sq(&self, sq: SQ) -> Option<Player> {
        assert!(sq < 64);
        let bb: BitBoard = sq_to_bb(sq);
        if bb & self.occupied_black() != 0 { return Some(Player::Black)}
        if bb & self.occupied_white() != 0 { return Some(Player::White)}
        None
    }

    // Returns the player, if any, at the square
    pub fn player_at_sq(&self, s: SQ) -> Option<Player> {
        let bb = sq_to_bb(s);
        if self.occupied_white() & bb != 0 {
            return Some(Player::White);
        } else if self.occupied_black() & bb != 0{
            return Some(Player::Black);
        }
        None
    }

    // Returns the square of the King for a given player
    pub fn king_sq(&self, player: Player) -> SQ {
        bb_to_sq(self.bit_boards[player as usize][Piece::K as usize])
    }

    // Returns the pinned pieces of player
    // Pinned being defined as pinned to the king
    pub fn pinned_pieces(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }
}

// Castling
impl  Board  {
    pub fn can_castle(&self, player: Player) -> bool {
        unimplemented!();
    }
}

// Checking
impl  Board  {

    // If current side to move is in check
    pub fn in_check(&self) -> bool {
        self.state.checkers_bb != 0
    }

    // Checks on the current player's king
    pub fn checkers(&self) -> BitBoard {
        self.state.checkers_bb
    }

    // Pieces the current side can move to discover check
    pub fn discovered_check_candidates(&self) -> BitBoard {
        self.state.blockers_king[other_player(self.turn) as usize] & self.get_occupied_player(self.turn)
    }

    // Gets the Pinned pieces for the given player
    pub fn pieces_pinned(&self, player: Player) -> BitBoard {
        self.state.blockers_king[player as usize] & self.get_occupied_player(player)
    }
}


impl  Board  {
    // Attacks to / From a given square
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

    // Tests if a given move is legal
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
        (self.pinned_pieces(self.turn) & src_bb == 0) || self.magic_helper.aligned(src,dst, self.king_sq(self.turn))
    }

    // Used to check for Hashing errors from TT Tables
    pub fn pseudo_legal_move(&self, m: BitMove) -> bool {
        unimplemented!();
    }

    // Checks if a move will give check to the opposing player's King
    // I am too drunk to be making this right now
    pub fn gives_check(&self, m: BitMove) -> bool {
        let src: SQ = m.get_src();
        let dst: SQ = m.get_dest();
        let src_bb: BitBoard = sq_to_bb(src);
        let dst_bb: BitBoard = sq_to_bb(dst);
        let opp_king_sq: SQ = self.king_sq(other_player(self.turn));

        assert_ne!(src, dst);
        assert_eq!(self.color_of_sq(src).unwrap(),self.turn);

        // Direct check mother fuckas
        if self.state.check_sqs[self.piece_at_sq(src).unwrap() as usize] & dst_bb != 0 {
            return true;
        }

        // Discovered check mother fuckas
        if (self.discovered_check_candidates() & src_bb != 0)
            && !self.magic_helper.aligned(src, dst, opp_king_sq) {
            return true;
        }

        match m.move_type() {
            MoveType::Normal => return false,
            MoveType::Promotion => {
                let attacks_bb = match m.promo_piece() {
                    Piece::N => self.magic_helper.knight_moves(dst),
                    Piece::B => self.magic_helper.bishop_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::R => self.magic_helper.rook_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::Q => self.magic_helper.queen_moves(self.get_occupied() ^ src_bb, dst),
                    _ => panic!()
                };
                return attacks_bb & sq_to_bb(opp_king_sq) != 0
            },
            MoveType::EnPassant => {
                let captured_sq: SQ = make_sq(file_of_sq(dst), rank_of_sq(src));
                let b: BitBoard = (self.get_occupied() ^ src_bb ^ sq_to_bb(captured_sq)) | dst_bb;

                let turn_sliding_p: BitBoard = self.sliding_piece_bb(self.turn);
                let turn_diag_p: BitBoard = self.diagonal_piece_bb(self.turn);

                return (self.magic_helper.rook_moves(b, opp_king_sq) | turn_sliding_p)
                    & (self.magic_helper.bishop_moves(b, opp_king_sq) | turn_diag_p) != 0;
            },
            MoveType::Castle => {
                let k_from: SQ = src;
                let r_from: SQ = dst;

                let k_to: SQ = relative_square(self.turn, {
                    if r_from > k_from { 6 } else { 2 }
                });
                let r_to: SQ = relative_square(self.turn, {
                    if r_from > k_from { 5 } else { 3 }
                });

                return (self.magic_helper.rook_moves(0, r_to) & sq_to_bb(opp_king_sq) != 0)
                    && (self.magic_helper.rook_moves(self.get_occupied() ^ sq_to_bb(k_from) ^ sq_to_bb(r_from), opp_king_sq)) != 0;
            },
            MoveType::Normal => { return false; }
        }
        unreachable!();
    }

    // Returns the piece that was moved
    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
        self.piece_at_sq(src).unwrap()
    }

    // Returns the piece that was captured, if any
    pub fn captured_piece(&self, m: BitMove) -> Piece {
        let dst = m.get_dest();
        self.piece_at_bb(sq_to_bb(dst),other_player(self.turn)).unwrap()
    }

}

// Printing and Debugging Functions
impl  Board  {

    // Returns a prettified String of the current board
    pub fn pretty_string(&self) -> String {
        unimplemented!();
    }

    // prints a prettified representation of the board
    pub fn pretty_print(&self) {
        unimplemented!();
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
        self.piece_at_sq(self.king_sq(Player::White)).unwrap() == Piece::K
        && self.piece_at_sq(self.king_sq(Player::Black)).unwrap() == Piece::K
        && (self.state.ep_square == 0 || self.state.ep_square == 64
            || relative_rank(self.turn,self.state.ep_square) != 5)
    }

    fn check_king(&self) -> bool {
        // TODO: Implement attacks to opposing king must be zero
        self.count_piece(Player::White, Piece::K,) == 1
        &&  self.count_piece(Player::Black, Piece::K) == 1

    }

    fn check_bitboards(&self) -> bool {
        if self.occupied_white() & self.occupied_black() != 0
        || (self.occupied_white() | self.occupied_white()) != self.get_occupied() {
            return false;
        }
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






