use templates::Piece as Piece;
use templates::Player as Player;
use templates::*;
use magic_helper::MagicHelper;
use bit_twiddles::*;
use piece_move::BitMove;
use fen;
use movegen;
use std::sync::Arc;




#[derive(Copy, Clone)]
pub struct BitBoardStates {
    pub w_pawn: BitBoard,
    pub w_knight: BitBoard,
    pub w_bishop: BitBoard,
    pub w_rook: BitBoard,
    pub w_queen: BitBoard,
    pub w_king: BitBoard,
    pub w_occ: BitBoard,
    pub b_pawn: BitBoard,
    pub b_knight: BitBoard,
    pub b_bishop: BitBoard,
    pub b_rook: BitBoard,
    pub b_queen: BitBoard,
    pub b_king: BitBoard,
    pub b_occ: BitBoard
}


impl BitBoardStates {
    fn blank() -> BitBoardStates {
        BitBoardStates {
            w_pawn: 0,
            w_knight: 0,
            w_bishop: 0,
            w_rook: 0,
            w_queen: 0,
            w_king: 0,
            w_occ: 0,
            b_pawn: 0,
            b_knight: 0,
            b_bishop: 0,
            b_rook: 0,
            b_queen: 0,
            b_king: 0,
            b_occ: 0
        }
    }

    // Creates occupancy bb from other bb;
    pub fn init_occ_bb(&mut self) {
        let b__occ: BitBoard = self.b_pawn | self.b_knight | self.b_bishop | self.b_rook | self.b_queen | self.b_king;
        let w__occ: BitBoard = self.w_pawn | self.w_knight | self.w_bishop | self.w_rook | self.w_queen | self.w_king;
        self.b_occ = b__occ;
        self.w_occ = w__occ;
    }
}

// Returns blank bitboards
impl Default for BitBoardStates {
    fn default() -> BitBoardStates {
        BitBoardStates {
            w_pawn: 0b0000000000000000000000000000000000000000000000001111111100000000,
            w_knight: 0b0000000000000000000000000000000000000000000000000000000001000010,
            w_bishop: 0b0000000000000000000000000000000000000000000000000000000000100100,
            w_rook: 0b0000000000000000000000000000000000000000000000000000000010000001,
            w_queen: 0b0000000000000000000000000000000000000000000000000000000000001000,
            w_king: 0b0000000000000000000000000000000000000000000000000000000000010000,
            w_occ: 0b0000000000000000000000000000000000000000000000001111111111111111,
            b_pawn: 0b0000000011111111000000000000000000000000000000000000000000000000,
            b_knight: 0b0100001000000000000000000000000000000000000000000000000000000000,
            b_bishop: 0b0010010000000000000000000000000000000000000000000000000000000000,
            b_rook: 0b1000000100000000000000000000000000000000000000000000000000000000,
            b_queen: 0b0000100000000000000000000000000000000000000000000000000000000000,
            b_king: 0b0001000000000000000000000000000000000000000000000000000000000000,
            b_occ: 0b1111111111111111000000000000000000000000000000000000000000000000,
        }
    }
}



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

    //  castling      ->  0000WWBB, left = 1 -> king side castle available, right = 1 -> queen side castle available
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
            castling: 0,
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
}


#[derive(Clone)]
pub struct Board<'a,'b>  {
    // Basic information
    pub turn: Player,
    pub bit_boards: BitBoardStates,
    pub half_moves: u16, // Total moves
    pub depth: u8, // current depth from actual position

    // State of the Board
    pub state: BoardState,

    // Not copied
    pub undo_moves: Vec<BitMove>,
    pub move_states: Vec<BoardState>,

    // Special Case
    pub magic_helper: Arc<MagicHelper <'a,'b>>,
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
impl <'a, 'b> Board <'a, 'b> {

    // Default, starting board
    pub fn default() -> Board<'a, 'b> {
        let mut b = Board {
            turn: Player::White,
            bit_boards: BitBoardStates::default(),
            half_moves: 0,
            depth: 0,
            state: BoardState::default(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: Arc::new(MagicHelper::new())
        };
        b.set_zob_hash();
        b
    }

    // Simple Version for testing, Skips creation of MagicHelper
    pub fn simple() -> Board<'a, 'b> {
        Board {
            turn: Player::White,
            bit_boards: BitBoardStates::default(),
            half_moves: 0,
            depth: 0,
            state: BoardState::default(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: Arc::new(MagicHelper::simple())
        }
    }

    // Creates a new board from an already created MagicHelper
    pub fn new(m_help: &Arc<MagicHelper<'a, 'b>>) -> Board<'a, 'b> {
        let mut b = Board {
            turn: Player::White,
            bit_boards: BitBoardStates::default(),
            half_moves: 0,
            depth: 0,
            state: BoardState::default(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: m_help.clone()
        };
        b.set_zob_hash();
        b
    }

    // Returns Shallow clone of current board with no Past Move List
    pub fn shallow_clone(&self) -> Board {
        Board {
            turn: self.turn,
            bit_boards: self.bit_boards.clone(),
            half_moves: self.half_moves,
            depth: self.depth,
            state: self.state.clone(),
            undo_moves: Vec::new(),
            move_states: Vec::new(),
            magic_helper: self.magic_helper.clone()
        }
    }

    // https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
    // "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",
    // Creates a new Board from a fen string
    pub fn new_from_fen(fen: String) -> Result<Board<'a, 'b>, String> {
        unimplemented!();
//        fen::generate_board(fen)
    }
}

// Public Move Gen & Mutation Functions
impl <'a, 'b> Board <'a, 'b> {

    // Applies the bitmove to the board
    pub fn apply_move(&mut self, bit_move: BitMove) {
        unimplemented!();
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
impl <'a, 'b> Board <'a, 'b> {
    fn put_piece(&mut self) {
        unimplemented!();
    }

    fn remove_piece(&mut self) {
        unimplemented!();
    }

    fn move_piece(&mut self) {
        unimplemented!();
    }

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

        self.state.check_sqs[Piece::P  as usize] = self.magic_helper.pawn_attacks_from(ksq,other_player(self.turn));
        self.state.check_sqs[Piece::N  as usize] = self.magic_helper.knight_moves(ksq);
        self.state.check_sqs[Piece::B  as usize] = self.magic_helper.bishop_moves(occupied, ksq);
        self.state.check_sqs[Piece::R  as usize] = self.magic_helper.rook_moves(occupied, ksq);
        self.state.check_sqs[Piece::Q  as usize] = self.state.check_sqs[Piece::B  as usize]
                                                 | self.state.check_sqs[Piece::R  as usize];
        self.state.check_sqs[Piece::K  as usize] = 0;
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




// Zobrist
impl <'a, 'b> Board <'a, 'b> {

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
            zob ^= self.magic_helper.zobrist.sq_piece[piece.unwrap() as usize][sq as usize];
        }
        let ep = self.state.ep_square;
        if ep != 0 && ep < 64 {
            let file: u8 = file_of_sq(ep);
            zob ^= self.magic_helper.zobrist.en_p[file as usize];
        }

        match self.turn {
            Player::Black =>  zob ^= self.magic_helper.zobrist.side,
            Player::White => {}
        }

        let c = self.state.castling;
        assert!((c as usize) < CASTLING_CNT);
        zob ^= self.magic_helper.zobrist.castle[c as usize];
        self.state.zobrast = zob;
    }
}

// Position Representation
impl <'a, 'b> Board <'a, 'b> {
    // Gets all occupied Squares
    pub fn get_occupied(&self) -> BitBoard {
        self.bit_boards.w_occ | self.bit_boards.b_occ
    }

    // Get the BitBoard of the squares occupied by player
    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.occupied_white(),
            Player::Black => self.occupied_black(),
        }
    }

    // Returns a Bitboard consisting of only the squares occupied by the White Player
    pub fn occupied_white(&self) -> BitBoard { self.bit_boards.w_occ }

    // Returns a BitBoard consisting of only the squares occupied by the Black Player
    pub fn occupied_black(&self) -> BitBoard { self.bit_boards.b_occ }

    // Returns Bitboard for one Piece and One Player
    pub fn piece_bb(&self, player: Player, piece: Piece) -> BitBoard {
        match player {
            Player::White => {
                match piece {
                    Piece::K => (self.bit_boards.w_king),
                    Piece::Q => (self.bit_boards.w_queen),
                    Piece::R => (self.bit_boards.w_rook),
                    Piece::B => (self.bit_boards.w_bishop),
                    Piece::N => (self.bit_boards.w_knight),
                    Piece::P => (self.bit_boards.w_pawn),
                }
            }
            Player::Black => {
                match piece {
                    Piece::K => (self.bit_boards.b_king),
                    Piece::Q => (self.bit_boards.b_queen),
                    Piece::R => (self.bit_boards.b_rook),
                    Piece::B => (self.bit_boards.b_bishop),
                    Piece::N => (self.bit_boards.b_knight),
                    Piece::P => (self.bit_boards.b_pawn),
                }
            }
        }
    }

    // Horizontally moving and Vertically moving pieces of player (Queens and Rooks)
    pub fn sliding_piece_bb(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.bit_boards.w_queen ^ self.bit_boards.w_rook,
            Player::Black => self.bit_boards.b_queen ^ self.bit_boards.b_rook,
        }
    }
    // reutns BitBoard of Diagonal moving pieces (Queens and Bishops)
    pub fn diagonal_piece_bb(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.bit_boards.w_queen ^ self.bit_boards.w_bishop,
            Player::Black => self.bit_boards.b_queen ^ self.bit_boards.b_bishop,
        }
    }

    // Bitboard of the pieces of both sides
    pub fn piece_bb_both_players(&self, piece: Piece) -> BitBoard {
        match piece {
            Piece::P => self.bit_boards.b_pawn | self.bit_boards.w_pawn,
            Piece::N => self.bit_boards.b_knight | self.bit_boards.w_knight,
            Piece::B => self.bit_boards.b_bishop | self.bit_boards.w_bishop,
            Piece::R => self.bit_boards.b_rook | self.bit_boards.w_rook,
            Piece::Q => self.bit_boards.b_queen | self.bit_boards.w_queen,
            Piece::K => self.bit_boards.b_king | self.bit_boards.w_king,
        }
    }

    pub fn piece_two_bb_both_players(&self, piece: Piece, piece2: Piece) -> BitBoard {
        self.piece_bb_both_players(piece) | self.piece_bb_both_players(piece2)
    }

    // Total number of pieces of type Piece and of player P
    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        let x = self.piece_bb(player, piece);
        popcount64(x)
    }

    // Total number of pieces of Player
    pub fn count_pieces_player(&self, player: Player) -> u8 {
        popcount64(self.get_occupied_player(player))
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
        if self.piece_bb(Player::White, Piece::P) & src_bit != 0 { return Some(Piece::P) };
        if self.piece_bb(Player::White, Piece::R) & src_bit != 0 { return Some(Piece::R) };
        if self.piece_bb(Player::White, Piece::N) & src_bit != 0 { return Some(Piece::N) };
        if self.piece_bb(Player::White, Piece::Q) & src_bit != 0 { return Some(Piece::Q) };
        if self.piece_bb(Player::White, Piece::B) & src_bit != 0 { return Some(Piece::B) };
        if self.piece_bb(Player::White, Piece::K) & src_bit != 0 { return Some(Piece::K) };
        if self.piece_bb(Player::Black, Piece::P) & src_bit != 0 { return Some(Piece::P) };
        if self.piece_bb(Player::White, Piece::R) & src_bit != 0 { return Some(Piece::R) };
        if self.piece_bb(Player::Black, Piece::N) & src_bit != 0 { return Some(Piece::N) };
        if self.piece_bb(Player::Black, Piece::Q) & src_bit != 0 { return Some(Piece::Q) };
        if self.piece_bb(Player::Black, Piece::B) & src_bit != 0 { return Some(Piece::B) };
        if self.piece_bb(Player::Black, Piece::K) & src_bit != 0 { return Some(Piece::K) };
        None
    }

    // Returns the Piece, if any, at the square
    pub fn piece_at_sq(&self, sq: SQ)-> Option<Piece> {
        assert!(sq < 64);
        self.piece_at_bb_all(sq_to_bb(sq))
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
        match player {
            Player::White => bb_to_sq(self.bit_boards.w_king),
            Player::Black => bb_to_sq(self.bit_boards.b_king),
        }
    }
}

// Castling
impl <'a, 'b> Board <'a, 'b> {
    pub fn can_castle(&self, player: Player) -> bool {
        unimplemented!();
    }
}

// Checking
impl <'a, 'b> Board <'a, 'b> {

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


impl <'a, 'b> Board <'a, 'b> {
    // Attacks to / From a given square
    pub fn attackers_to(&self, sq: SQ) -> BitBoard {
        let occupied: BitBoard = self.get_occupied();
        unimplemented!();
    }

}

// Move Testing
impl <'a, 'b> Board <'a, 'b> {
    pub fn legal_move(&self, m: BitMove) -> bool {
        unimplemented!();
    }
    pub fn pseudo_legal_move(&self, m: BitMove) -> bool {
        unimplemented!();
    }

    // Checks if a move will give check to the opposing player's King
    pub fn gives_check(&self, m: BitMove) -> bool {
        let src: SQ = m.get_src();
        let dst: SQ = m.get_dest();
        assert_ne!(src, dst);
        assert_eq!(self.color_of_sq(src),self.turn);




    }

    pub fn moved_piece(&self, m: BitMove) -> Piece {
        let src = m.get_src();
    }

    pub fn captured_piece(&self, m: BitMove) -> Piece {
        unimplemented!();
    }

}

// Printing and Debugging Functions
impl <'a, 'b> Board <'a, 'b> {

    // Returns a prettified String of the current board
    pub fn pretty_string(&self) -> String {
        unimplemented!();
    }

    // prints a prettified representation of the board
    pub fn pretty_print(&self) {
        unimplemented!();
    }

    // Checks the current state of the Board
    pub fn is_okay(&self) -> bool {
        unimplemented!();
        true
    }
}



