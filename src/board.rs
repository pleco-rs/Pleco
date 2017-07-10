use templates::*;
use magic_helper::MagicHelper;
use bit_twiddles::*;
use piece_move::{BitMove,MoveType};
use fen;
use std::option::*;
use std::mem;
use std::sync::Arc;




// Initialize MAGIC_HELPER as a static structure

lazy_static! {
    pub static ref MAGIC_HELPER: MagicHelper<'static,'static> = MagicHelper::new();
}

// ***** BOARD STATE ***** //

// TODO: change casting bits to a type

// 'BoardState' is a structure to hold useful information concerning the current state of the board
// This is information that is computed upon making a move, and requires expensive computation at that.
// It is stored in the Heap by 'Board' as an Arc<BoardState>, as cloning the board can lead to multiple
// references to the same BoardState.
//
// Contains a 'prev' field to point to the BoardState Before the current one. Used as a 'Option<Arc<BoardState>>'
// in order to account for the possibility of their being no previous move.
//

#[derive(Clone)]
pub struct BoardState {
    // Automatically Created
    pub castling: u8, // special castling bits
    pub rule_50: i8,
    pub ply: u8,
    pub ep_square: SQ,

    // Recomputed after a move
    pub zobrast: u64,
    pub captured_piece: Option<Piece>,
    pub checkers_bb: BitBoard, // What squares is the current player receiving check from?
    pub blockers_king: [BitBoard; PLAYER_CNT],
    pub pinners_king: [BitBoard; PLAYER_CNT],
    pub check_sqs: [BitBoard; PIECE_CNT],

    // Previous state
    pub prev: Option<Arc<BoardState>>,

    //  castling      ->  0000WWBB, left = 1 -> king side castle possible, right = 1 -> queen side castle possible
    //  rule50        -> 50 moves without capture, for draws
    //  ply           -> How many moves deep this current thread is
    //  ep_square     -> square of en-passant, if any
    //  zobrast       -> zobrist key
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
            prev: None,
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
            prev: None, // NOTE, WHEN CLONING THE SUPER METHOD HAS TO MANUALLY CHANGE THIS
        }
    }
}





// ***** BOARD ***** //

// 'Board' is the big daddy of information. Contains everything that needs to be known about the current
// state of the Game. It is used by both Engines and Players alike, with the Engines (obviously) containing
// the original copy of the Board. (Shallow) Copying the Board creates a copy with all the neccesary information
// about the current state, but doesn't include information about the previous states.
//
// BitBoards are stored in the following format (as in bit 0 is Square A1, bit 1 is B1, etc
//
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


pub struct Board {
    // Basic information
    pub turn: Player, // Current turn
    pub bit_boards: [[BitBoard; PIECE_CNT]; PLAYER_CNT], // Occupancy per player per piece
    pub occ: [BitBoard; PLAYER_CNT], // Occupancy per Player
    pub occ_all: BitBoard, // Total Occupancy BB
    pub half_moves: u16, // Total moves
    pub depth: u8, // current depth from actual position (Basically, moves since shallow clone was called)
    pub piece_counts: [[u8; PIECE_CNT]; PLAYER_CNT],
    pub piece_locations: PieceLocations,
    
    // State of the Board, Un modifiable.
    // Arc to allow easy and quick copying of boards without copying memory
    // or recomputing BoardStates.
    pub state: Arc<BoardState>,

    // List of Moves that have been played so far
    // undo_moves.len() == depth, as undo_moves is not copied upon
    // shallow clone
    pub undo_moves: Vec<BitMove>,

    // Special Case
    pub magic_helper: &'static MAGIC_HELPER,
}


// Initializers!
impl Board {

    // Default, starting position of the board
    pub fn default() -> Board {
        let mut b = Board {
            turn: Player::White,
            bit_boards: return_start_bb(),
            occ: [START_WHITE_OCC, START_BLACK_OCC],
            occ_all: START_OCC_ALL,
            half_moves: 0,
            depth: 0,
            piece_counts: [[0; PIECE_CNT]; PLAYER_CNT],
            piece_locations: PieceLocations::default(),
            state: Arc::new(BoardState::default()),
            undo_moves: Vec::with_capacity(100), // As this is the main board, might as well reserve space
            magic_helper: &MAGIC_HELPER
        };
        b.set_zob_hash();
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
            piece_counts: self.piece_counts.clone(),
            piece_locations: self.piece_locations.clone(),
            state: self.state.clone(),
            undo_moves: Vec::with_capacity(16), // 32 Bytes taken up
            magic_helper: &MAGIC_HELPER,
        }
    }

    // Returns an EXACT memory representation of that Board.
    // unsafe as this is to only be used by engines, and is more computationally expensive
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
            undo_moves: self.undo_moves.clone(), // 32 Bytes taken up
            magic_helper: &MAGIC_HELPER,
        }
    }

    // Helper method for setting the piece states on initilization.
    // Really only used when creating the Board from scratch, rather than copying
    fn set_piece_states(&mut self) {
        for player in ALL_PLAYERS.iter() {
            for piece in ALL_PIECES.iter() {
                self.piece_counts[*player as usize][*piece as usize] = popcount64(self.piece_bb(*player,*piece));
            }
        }

        for square in 0..SQ_CNT as u8 {
            let bb = sq_to_bb(square);
            if bb & self.get_occupied() != 0 {
                let player = if bb & self.occupied_black() == 0 { Player::White } else { Player::Black };
                let piece = if self.piece_bb(player, Piece::P) & bb != 0 { Piece::P }
                    else if self.piece_bb(player, Piece::N) & bb != 0 { Piece::N }
                        else if self.piece_bb(player, Piece::B) & bb != 0 { Piece::B }
                            else if self.piece_bb(player, Piece::R) & bb != 0 { Piece::R }
                                else if self.piece_bb(player, Piece::Q) & bb != 0 { Piece::Q }
                                    else if self.piece_bb(player, Piece::K) & bb != 0 { Piece::K } else { panic!() };
            self.piece_locations.place(square,player,piece);
            } else {
                self.piece_locations.remove(square);
            }
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

    // Applies the BitMove to the Board
    pub fn apply_move(&mut self, bit_move: BitMove) {

        // Check for stupidity
        assert_ne!(bit_move.get_src(),bit_move.get_dest());

        // Does this move give check?
        let gives_check: bool = self.gives_check(bit_move);

        let mut zob: u64 = self.state.zobrast ^ self.magic_helper.zobrist.side;

        // New Arc for the board to have by making a partial clone of the current state
        let mut next_arc_state = Arc::new(self.state.partial_clone());

        {
            // Seperate Block to allow derefencing the state
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
                self.piece_at_sq(from)
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
                self.apply_castling(us, to, from, &mut k_to, &mut r_to);
                zob ^= self.magic_helper.z_piece_at_sq(Piece::R,k_to) ^ self.magic_helper.z_piece_at_sq(Piece::R,r_to);
                new_state.captured_piece = None;
                // TODO: Set castling rights Zobrist
                new_state.castling &= !CASTLE_RIGHTS[us as usize];
            }

            // A piece has been captured
            if captured.is_some() {
                let mut cap_sq: SQ = to;
                let cap_p: Piece = captured.unwrap(); // This shouldn't panic unless move is void
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

                // Reset Rule 50
                new_state.rule_50 = 0;
            }

            // Update hash for moving piece
            zob ^= self.magic_helper.z_piece_at_sq(piece,to) ^ self.magic_helper.z_piece_at_sq(piece,from);

            if self.state.ep_square != NO_SQ {
                zob ^= self.magic_helper.z_ep_file(self.state.ep_square);
                new_state.ep_square = NO_SQ;
            }

            // Update castling rights
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

            // Actually move the piece
            if !bit_move.is_castle() && !bit_move.is_promo() {
                self.move_piece_c(piece, to, from, us);
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
                new_state.checkers_bb = self.attackers_to(self.king_sq(them),self.get_occupied());
            }

            self.turn = them;
            self.undo_moves.push(bit_move);
            self.set_check_info(new_state); // Set the checking information
        }
        self.state = next_arc_state;
        assert!(self.is_okay());
    }

    pub fn undo_move(&mut self) {
        assert!(self.undo_moves.len() > 0);
        assert!(self.state.prev.is_some());

        self.turn = other_player(self.turn);

        let us: Player = self.turn;

        unimplemented!();
    }

    pub fn generate_moves(&self) -> Vec<BitMove> {
        unimplemented!();
    }

    pub fn generate_moves_of_type(&self, gen_type: GenTypes) -> Vec<BitMove> {
        unimplemented!();
    }
}

// Private Mutating Functions
impl  Board  {

    // After a move is made, Information about the checking situation is created
    fn set_check_info(&self, board_state: &mut BoardState) {
        let mut white_pinners = 0;
        board_state.blockers_king[Player::White as usize]  = {
            self.slider_blockers(self.occupied_black(), self.king_sq(Player::White), &mut white_pinners) };
        board_state.pinners_king[Player::White as usize] = white_pinners;

        let mut black_pinners = 0;
        board_state.blockers_king[Player::Black as usize]  = {
            self.slider_blockers(self.occupied_white(), self.king_sq(Player::Black), &mut black_pinners) };
        board_state.pinners_king[Player::Black as usize] = black_pinners;

        let ksq: SQ = self.king_sq(other_player(self.turn));
        let occupied = self.get_occupied();

        board_state.check_sqs[Piece::P as usize] = self.magic_helper.pawn_attacks_from(ksq,other_player(self.turn));
        board_state.check_sqs[Piece::N as usize] = self.magic_helper.knight_moves(ksq);
        board_state.check_sqs[Piece::B as usize] = self.magic_helper.bishop_moves(occupied, ksq);
        board_state.check_sqs[Piece::R as usize] = self.magic_helper.rook_moves(occupied, ksq);
        board_state.check_sqs[Piece::Q as usize] = self.state.check_sqs[Piece::B as usize]
                                                 | self.state.check_sqs[Piece::R as usize];
        board_state.check_sqs[Piece::K  as usize] = 0;
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

        self.piece_locations.place(square,player,piece);
        self.piece_counts[player as usize][piece as usize] += 1;
        // Note: Should We set captured Piece?
    }

    // remove a piece, color is known
    fn remove_piece_c(&mut self, piece: Piece, square: SQ, player: Player) {
        assert_eq!(self.piece_at_sq(square).unwrap(),piece);
        let bb = sq_to_bb(square);
        self.occ_all ^= bb;
        self.occ[player as usize] ^= bb;
        self.bit_boards[player as usize][piece as usize] ^= bb;

        self.piece_locations.remove(square);
        self.piece_counts[player as usize][piece as usize] -= 1;
    }

    // move a piece, color is known
    fn move_piece_c(&mut self, piece: Piece, from: SQ, to: SQ, player: Player) {
        assert_ne!(from, to);
        assert_eq!(self.piece_at_sq(from).unwrap(),piece);
        let comb_bb = sq_to_bb(from) | sq_to_bb(to);

        self.occ_all ^= comb_bb;
        self.occ[player as usize] ^= comb_bb;
        self.bit_boards[player as usize][piece as usize] ^= comb_bb;

        self.piece_locations.remove(from);
        self.piece_locations.place(to,player,piece);
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
        let k_dst: SQ = self.king_sq(player);
        let king_side: bool = k_src < r_src;
        let r_dst: SQ = if king_side {
            relative_square(player,7)
        } else {
            relative_square(player,0)
        };

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
        };

        Arc::get_mut(&mut self.state).unwrap().zobrast = zob;
    }



}

// Position Representation
impl  Board  {
    // Gets all occupied Squares
    pub fn get_occupied(&self) -> BitBoard {
        self.occ_all
    }

    // Get the BitBoard of the squares occupied by player
    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        self.occ[player as usize]
    }

    // Returns a Bitboard consisting of only the squares occupied by the White Player
    pub fn occupied_white(&self) -> BitBoard {
        self.occ[Player::White as usize]
    }

    // Returns a BitBoard consisting of only the squares occupied by the Black Player
    pub fn occupied_black(&self) -> BitBoard {
        self.occ[Player::Black as usize]
    }

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
        self.piece_counts[player as usize][piece as usize]
    }

    // Total number of pieces of Player
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        self.piece_counts[player as usize].iter().sum()
    }

    // Returns the piece at the given place. Number of bits must be equal to 1, or else won't work
    pub fn piece_at_bb(&self, src_bit: BitBoard, player: Player) -> Option<Piece> {
        let sq: SQ = bb_to_sq(src_bit);
        assert!(sq_is_okay(sq));
        self.piece_locations.piece_at_for_player(sq,player)
    }

    // Returns the piece at the given place. Number of bits must be equal to 1, or else won't work
    pub fn piece_at_bb_all(&self, src_bit: BitBoard)-> Option<Piece> {
        let square: SQ = bb_to_sq(src_bit);
        assert!(sq_is_okay(square));
        self.piece_locations.piece_at(square)
    }

    // Returns the Piece, if any, at the square
    pub fn piece_at_sq(&self, sq: SQ)-> Option<Piece> {
        assert!(sq < 64);
        self.piece_locations.piece_at(sq)
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
        self.piece_locations.player_at(s)
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
            MoveType::Normal => return false, // Nothing to check here
            MoveType::Promotion => { // check if the Promo Piece attacks king
                let attacks_bb = match m.promo_piece() {
                    Piece::N => self.magic_helper.knight_moves(dst),
                    Piece::B => self.magic_helper.bishop_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::R => self.magic_helper.rook_moves(self.get_occupied() ^ src_bb, dst),
                    Piece::Q => self.magic_helper.queen_moves(self.get_occupied() ^ src_bb, dst),
                    _ => unreachable!()
                };
                return attacks_bb & sq_to_bb(opp_king_sq) != 0
            },
            MoveType::EnPassant => {
                // Check for indirect check from the removal of the captured pawn
                let captured_sq: SQ = make_sq(file_of_sq(dst), rank_of_sq(src));
                let b: BitBoard = (self.get_occupied() ^ src_bb ^ sq_to_bb(captured_sq)) | dst_bb;

                let turn_sliding_p: BitBoard = self.sliding_piece_bb(self.turn);
                let turn_diag_p: BitBoard = self.diagonal_piece_bb(self.turn);

                // TODO: is this right?
                return (self.magic_helper.rook_moves(b, opp_king_sq) | turn_sliding_p)
                    & (self.magic_helper.bishop_moves(b, opp_king_sq) | turn_diag_p) != 0;
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

                return (self.magic_helper.rook_moves(0, r_to) & sq_to_bb(opp_king_sq) != 0)
                    && (self.magic_helper.rook_moves(self.get_occupied() ^ sq_to_bb(k_from) ^ sq_to_bb(r_from), opp_king_sq)) != 0;
            }
        }
    }

    // Returns the piece that was moved
    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
        self.piece_at_sq(src).unwrap() // panics if no piece here :)
    }

    // Returns the piece that was captured, if any
    pub fn captured_piece(&self, m: BitMove) -> Option<Piece> {
        let dst = m.get_dest();
        self.piece_at_bb(sq_to_bb(dst),other_player(self.turn))
    }

}

// Printing and Debugging Functions
impl  Board  {

    // Returns a prettified String of the current board
    // Capital Letters are WHITE, lowe case are BLACK
    pub fn pretty_string(&self) -> String {
        let mut s = String::with_capacity(SQ_CNT * 2 + 8);
        for sq in SQ_DISPLAY_ORDER.iter() {
            let op = self.piece_locations.player_piece_at(sq.clone());
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

    // prints a prettified representation of the board
    pub fn pretty_print(&self) {
        println!("{}",self.pretty_string());
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


// 000 -> P
// 001 -> N
// 010 -> B
// 011 -> R
// 100 -> Q
// 101 -> K
// 110 -> ?
// 111 -> None
//
// 0xxx -> White
// 1xxx -> Black

// 1 byte per square
// 1 byte * 64 squares = 64 bytes
// 64 bytes / 2 bytes a u8 = 32 u8s

// TODO: Consider replacing with an array 64 long of bytes, rather thsn compressing each byte into two moves
//      Space vs. Lookup speed difference

// Struct to allow fast lookups for any square. Answers questions such as
// What Color if any is at the square? What Piece of any is at the square? etc
// Piece Locationsis a BLIND structure, Providing a function of  |sq| -> |Piece AND/OR Player|
// You cannot do the reverse, Looking up squares from a piece / player
#[derive(Clone,PartialEq,Copy)]
pub struct PieceLocations {
    data: [u8; 32]
}

// [0] = 00001111 --- (1,0)
// [1] = 00001111 --- (3,2)


impl PieceLocations {
    pub fn default() -> PieceLocations { PieceLocations {data: [0;32]}}
    pub fn place(&mut self, square: SQ, player: Player, piece: Piece, ) {
        assert!(sq_is_okay(square));
        let idx: usize = (square / 2) as usize;
        let rem: u8 = square % 2;
        let mask: u8 = {
            if rem == 0 {0x0F}
            else {0xF0}
        };
        self.data[idx] &= !mask;
        self.data[idx] |= self.create_sq(player,piece,rem);
    }

    pub fn remove(&mut self, square: SQ) {
        assert!(sq_is_okay(square));
        let idx: usize = (square / 2) as usize;
        let rem: u8 = square % 2;
        let mask: u8 = {
            if rem == 0 {0x0F} else {0xF0}
        };
        let blank: u8 = {
            if rem == 0 {0b0111} else {0b0111 << 4}
        };
        self.data[idx] &= !mask;
        self.data[idx] |= blank;


    }

    pub fn piece_at(&self, square: SQ) -> Option<Piece> {
        let idx: usize = (square / 2) as usize;
        let rem: u8 = square % 2;
        let byte: u8 = self.get_half_byte(idx, rem) & 0b0111;
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

    pub fn piece_at_for_player(&self, square: SQ, player: Player) -> Option<Piece>{
        let op = self.player_piece_at(square);
        if op.is_some() {
            let p = op.unwrap();
            if p.0 == player {
                return Some(p.1);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn player_at(&self, square: SQ) -> Option<Player> {
        let idx: usize = (square / 2) as usize;
        let rem: u8 = square % 2;
        let mut byte: u8 = self.get_half_byte(idx, rem);
        if byte & 0b1000 == 0b0111 {
            return None;
        }
        byte &= 0b1000;
        if byte == 0 {
            return Some(Player::White);
        } else {
            return Some(Player::Black);
        }
    }

    pub fn player_piece_at(&self, square: SQ) -> Option<(Player,Piece)> {
        let idx: usize = (square / 2) as usize;
        let rem: u8 = square % 2;
        let byte: u8 = self.get_half_byte(idx, rem);
        match byte {
            0b0000 => Some((Player::White, Piece::P)),
            0b0001 => Some((Player::White, Piece::N)),
            0b0010 => Some((Player::White, Piece::B)),
            0b0011 => Some((Player::White, Piece::R)),
            0b0100 => Some((Player::White, Piece::Q)),
            0b0101 => Some((Player::White, Piece::K)),
            0b0110 => unreachable!(), // Undefined
            0b0111 => None,
            0b1000 => Some((Player::Black, Piece::P)),
            0b1001 => Some((Player::Black, Piece::N)),
            0b1010 => Some((Player::Black, Piece::B)),
            0b1011 => Some((Player::Black, Piece::R)),
            0b1100 => Some((Player::Black, Piece::Q)),
            0b1101 => Some((Player::Black, Piece::K)),
            0b1110 => unreachable!(), // Undefined
            0b1111 => None,
            _ => unreachable!()
        }
    }

    // Helper function to retrieve and shift the relevant 4 bits needed
    fn get_half_byte(&self, idx: usize, rem: u8) -> u8 {
        let bits = self.data[idx];
        if rem == 0 {
            return bits & 0x0F;
        } else {
            return bits.wrapping_shr(4);
        }
    }


    // Creates the 4 bits representing a piece and player
    fn create_sq(&self, player: Player, piece: Piece, rem: u8) -> u8 {
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
        if rem != 0 {
            return loc << 4;
        }
        loc
    }
}





