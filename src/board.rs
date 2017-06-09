use templates::Piece as Piece;
use templates::Player as Player;
use templates::*;
use magic_helper::MagicHelper;
use bit_twiddles::popcount64;
use piece_move::BitMove;
use fen;
use movegen;
use std::sync::Arc;





// Used for determining checks
#[derive(Copy, Clone)]
pub struct LastMoveData {
    pub piece_moved: Piece,
    pub src: SQ,
    pub dst: SQ,
}

#[derive(Copy, Clone)]
pub struct AllBitBoards {
    pub w_pawn: BitBoard,
    pub w_knight: BitBoard,
    pub w_bishop: BitBoard,
    pub w_rook: BitBoard,
    pub w_queen: BitBoard,
    pub w_king: BitBoard,
    pub b_pawn: BitBoard,
    pub b_knight: BitBoard,
    pub b_bishop: BitBoard,
    pub b_rook: BitBoard,
    pub b_queen: BitBoard,
    pub b_king: BitBoard,
}

#[derive(Clone)]
pub struct Board<'a,'b>  {
    pub bit_boards: AllBitBoards,
    pub turn: Player,
    pub depth: u16,
   // Tracks the depth of the current board, used by Bots
    pub castling: u8,
    // 0000WWBB, left = 1 -> king side castle available, right = 1 -> queen side castle available
    pub en_passant: SQ,
    // is the square of the enpassant unless equal to 2^64
    pub undo_moves: Vec<BitMove>,
    // Full list of undo-able moves
    pub ply: u8,
    // Tracks how many half-moves has been played so far
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

// FEN
impl <'a, 'b> Board <'a, 'b> {

    pub fn default() -> Board <'a, 'b> {
        Board {
            bit_boards: AllBitBoards::new(),
            turn: Player::White,
            depth: 0,
            castling: 0,
            en_passant: 0,
            undo_moves: Vec::new(),
            ply: 0,
            magic_helper: Arc::new(MagicHelper::new())
        }
    }

    pub fn new(m_help: &Arc<MagicHelper<'a, 'b>>) -> Board <'a, 'b> {
        Board  {
            bit_boards: AllBitBoards::new(),
            turn: Player::White,
            depth: 0,
            castling: 0,
            en_passant: 0,
            undo_moves: Vec::new(),
            ply: 0,
            magic_helper: m_help.clone()
        }
    }

    pub fn generate_moves(&self) -> Vec<BitMove> { movegen::get_moves(&self) }



    // https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
    // "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",
    pub fn new_from_fen(fen: String) -> Result<Board<'a, 'b>, String> {
        fen::generate_board(fen)
    }

    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        let x = self.get_bitboard(player, piece);
        popcount64(x)
    }

    pub fn count_pieces_player(&self, player: Player) -> u8 {
        popcount64(self.get_occupied_player(player))
    }

    // Returns Bitboard for one Piece and One Player
    pub fn get_bitboard(&self, player: Player, piece: Piece) -> BitBoard {
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

    // Returns set of all bit boards for that specific player
    pub fn get_bitboards_player(&self, player: Player) -> Vec<BitBoard> {
        let mut vector = Vec::with_capacity(6);
        match player {
            Player::White => {
                vector.push(self.bit_boards.w_king);
                vector.push(self.bit_boards.w_queen);
                vector.push(self.bit_boards.w_rook);
                vector.push(self.bit_boards.w_bishop);
                vector.push(self.bit_boards.w_knight);
                vector.push(self.bit_boards.w_pawn);
            }
            Player::Black => {
                vector.push(self.bit_boards.b_king);
                vector.push(self.bit_boards.b_queen);
                vector.push(self.bit_boards.b_rook);
                vector.push(self.bit_boards.b_bishop);
                vector.push(self.bit_boards.b_knight);
                vector.push(self.bit_boards.b_pawn);
            }
        };
        vector
    }

    // Gets all occupied Squares
    pub fn get_occupied(&self) -> BitBoard {
        self.occupied_black() | self.occupied_white()
    }

    // Returns Shallow clone of current board with no Past Move List
    pub fn shallow_clone(&self) -> Board {
        Board {
            bit_boards: AllBitBoards::new(),
            turn: self.turn,
            depth: self.depth,
            castling: self.castling,
            en_passant: self.en_passant,
            undo_moves: Vec::new(),
            ply: self.ply,
            magic_helper: self.magic_helper.clone()

        }
    }

    // Applies the bitmove to the board;
    pub fn apply_move(&mut self, bit_move: BitMove) {
        let us: Player = self.turn;
        let them: Player = match us {
            Player::White => Player::Black,
            Player::Black => Player::White
        };
        let src: SQ = bit_move.get_src();
        let dst: SQ = bit_move.get_dest();

        let src_bit: BitBoard = 1 << src;
        let dst_bit: BitBoard = 1 << dst;

        if bit_move.is_castle() {
            // IF CASTLE MOVE
            // White: King at index: 4
            // Black: King at index: 60
            if bit_move.is_king_castle() {
                // White: Rook at index: 7
                // Black: Rook at index: 63
                match us {
                    Player::White => {
                        let rook_pos: BitBoard = 1 << 7 | 1 << 5;
                        let king_pos: BitBoard = 1 << 4 | 1 << 6;
                        self.bit_boards.w_rook ^= rook_pos;
                        self.bit_boards.w_king ^= king_pos;
//                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 7, dst: 5 });
                    }
                    Player::Black => {
                        let rook_pos: BitBoard = 1 << 63 | 1 << 61;
                        let king_pos: BitBoard = 1 << 60 | 1 << 62;
                        self.bit_boards.b_rook ^= rook_pos;
                        self.bit_boards.b_king ^= king_pos;
//                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 63, dst: 61 });
                    }
                }
            } else {
                // White: Rook at index: 0
                // Black: Rook at index: 56
                match us {
                    Player::White => {
                        let rook_pos: BitBoard = 1      | 1 << 3;
                        let king_pos: BitBoard = 1 << 4 | 1 << 2;
                        self.bit_boards.w_rook ^= rook_pos;
                        self.bit_boards.w_king ^= king_pos;
//                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 0, dst: 3 });
                    }
                    Player::Black => {
                        let rook_pos: BitBoard = 1 << 56 | 1 << 59;
                        let king_pos: BitBoard = 1 << 60 | 1 << 58;
                        self.bit_boards.b_rook ^= rook_pos;
                        self.bit_boards.b_king ^= king_pos;
//                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 56, dst: 59 });
                    }
                }
            }
            match us {
                Player::White => { self.castling &= 0b11110011; }
                Player::Black => { self.castling &= 0b11111100; }
            }
        } else if bit_move.is_double_push().0 {
            // DOUBLE PAWN MOVE
            match us {
                Player::White => { self.bit_boards.w_pawn ^= src_bit | dst_bit; }
                Player::Black => { self.bit_boards.b_pawn ^= src_bit | dst_bit; }
            }
            self.en_passant = dst;
//            self.last_move = Some(LastMoveData { piece_moved: Piece::P, src: src, dst: dst });
        } else if bit_move.is_promo() {
            if bit_move.is_capture() {
                self.xor_bitboard_player_sq(them, dst_bit);
            }
            self.xor_bitboard_player_piece_sq(us, Piece::P, src_bit);
            self.xor_bitboard_player_piece_sq(us, bit_move.promo_piece(), dst_bit);
//            self.last_move = None;
        } else if bit_move.is_en_passant() {
            // PAWN ENPASSENT;
            match us {
                Player::White => {
                    self.bit_boards.w_pawn ^= src_bit | dst_bit;
                    self.bit_boards.b_pawn ^= dst_bit >> 8;
                }
                Player::Black => {
                    self.bit_boards.b_pawn ^= src_bit | dst_bit;
                    self.bit_boards.w_pawn ^= dst_bit << 8;
                }
            }
//            self.last_move = Some(LastMoveData { piece_moved: Piece::P, src: src, dst: dst });
        } else {
            // QUIET MOVE

            if bit_move.is_capture() { self.xor_bitboard_player_sq(them, dst_bit); }
            // check if capture, if so opponent board needs to be modified;

            // Modify own board
            let piece = self.get_piece_from_src(src_bit, us).unwrap();
            self.xor_bitboard_player_piece_sq(us, piece, src_bit | dst_bit);
//            self.last_move = Some( LastMoveData { piece_moved: piece, src: src, dst: dst } );
        }
        if !bit_move.is_double_push().0 { self.en_passant = 64; }

        self.ply += 1;
        self.turn = them;
    }

    // Returns the piece at the given place. Num bits src_bit == 1
    fn get_piece_from_src(&self, src_bit: BitBoard, player: Player) -> Option<Piece> {
        if self.get_bitboard(player, Piece::P) & src_bit != 0 { return Some(Piece::P) };
        if self.get_bitboard(player, Piece::R) & src_bit != 0 { return Some(Piece::R) };
        if self.get_bitboard(player, Piece::N) & src_bit != 0 { return Some(Piece::N) };
        if self.get_bitboard(player, Piece::Q) & src_bit != 0 { return Some(Piece::Q) };
        if self.get_bitboard(player, Piece::B) & src_bit != 0 { return Some(Piece::B) };
        if self.get_bitboard(player, Piece::K) & src_bit != 0 { return Some(Piece::K) };
        None
    }

    // XORs the Bitboard of (player,piece) by the input bit_board, figures out the piece & player itself
    fn xor_bitboard_sq(&mut self, square_bit: BitBoard) {
        let player = match self.get_occupied() & square_bit {
            0 => Player::Black,
            _ => Player::White
        };
        self.xor_bitboard_player_sq(player, square_bit);
    }

    // XORs the Bitboard of (player,piece) by the input bit_board, figures out the piece itself
    fn xor_bitboard_player_sq(&mut self, player: Player, square_bit: BitBoard) {
        let piece = self.get_piece_from_src(square_bit, player).unwrap();
        self.xor_bitboard_player_piece_sq(player, piece, square_bit);
    }

    // XORs the Bitboard of (piece, player) by the input bit_board
    fn xor_bitboard_player_piece_sq(&mut self, player: Player, piece: Piece, square_bit: BitBoard) {
        match player {
            Player::White => {
                match piece {
                    Piece::B => self.bit_boards.w_bishop ^= square_bit,
                    Piece::P => self.bit_boards.w_pawn ^= square_bit,
                    Piece::R => self.bit_boards.w_rook ^= square_bit,
                    Piece::N => self.bit_boards.w_knight ^= square_bit,
                    Piece::K => self.bit_boards.w_king ^= square_bit,
                    Piece::Q => self.bit_boards.w_queen ^= square_bit,
                };
            }
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.b_bishop ^= square_bit,
                    Piece::P => self.bit_boards.b_pawn ^= square_bit,
                    Piece::R => self.bit_boards.b_rook ^= square_bit,
                    Piece::N => self.bit_boards.b_knight ^= square_bit,
                    Piece::K => self.bit_boards.b_king ^= square_bit,
                    Piece::Q => self.bit_boards.b_queen ^= square_bit,
                };
            }
        };
    }

    // Sets the Bitboard of piece, player to parameter bit_board
    fn modifiy_bitboard(&mut self, bit_board: BitBoard, player: Player, piece: Piece) {
        match player {
            Player::White => {
                match piece {
                    Piece::B => self.bit_boards.w_bishop = bit_board,
                    Piece::P => self.bit_boards.w_pawn = bit_board,
                    Piece::R => self.bit_boards.w_rook = bit_board,
                    Piece::N => self.bit_boards.w_knight = bit_board,
                    Piece::K => self.bit_boards.w_king = bit_board,
                    Piece::Q => self.bit_boards.w_queen = bit_board,
                };
            }
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.b_bishop = bit_board,
                    Piece::P => self.bit_boards.b_pawn = bit_board,
                    Piece::R => self.bit_boards.b_rook = bit_board,
                    Piece::N => self.bit_boards.b_knight = bit_board,
                    Piece::K => self.bit_boards.b_king = bit_board,
                    Piece::Q => self.bit_boards.b_queen = bit_board,
                };
            }
        };
    }

    // Horizontally moving and Vertically moving pieves
    pub fn sliding_piece_bits(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.bit_boards.w_queen ^ self.bit_boards.w_rook,
            Player::Black => self.bit_boards.b_queen ^ self.bit_boards.b_rook,
        }
    }
    // Diagonal moving pieces
    pub fn diagonal_piece_bits(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.bit_boards.w_queen ^ self.bit_boards.w_bishop,
            Player::Black => self.bit_boards.b_queen ^ self.bit_boards.b_bishop,
        }
    }

    pub fn get_occupied_player(&self, player: Player) -> BitBoard {
        match player {
            Player::White => self.occupied_white(),
            Player::Black => self.occupied_black(),
        }
    }

    pub fn pretty_string(&self) -> String {
        unimplemented!();
    }

    pub fn pretty_print(&self) {
        unimplemented!();
    }

    pub fn occupied_white(&self) -> BitBoard {
        self.bit_boards.w_bishop
            | self.bit_boards.w_pawn
            | self.bit_boards.w_knight
            | self.bit_boards.w_rook
            | self.bit_boards.w_king
            | self.bit_boards.w_queen
    }

    pub fn occupied_black(&self) -> BitBoard {
        self.bit_boards.b_bishop
            | self.bit_boards.b_pawn
            | self.bit_boards.b_knight
            | self.bit_boards.b_rook
            | self.bit_boards.b_king
            | self.bit_boards.b_queen
    }
}

impl AllBitBoards {
    fn new() -> AllBitBoards {
        AllBitBoards {
            w_pawn: 0b0000000000000000000000000000000000000000000000001111111100000000,
            w_knight: 0b0000000000000000000000000000000000000000000000000000000001000010,
            w_bishop: 0b0000000000000000000000000000000000000000000000000000000000100100,
            w_rook: 0b0000000000000000000000000000000000000000000000000000000010000001,
            w_queen: 0b0000000000000000000000000000000000000000000000000000000000001000,
            w_king: 0b0000000000000000000000000000000000000000000000000000000000010000,
            b_pawn: 0b0000000011111111000000000000000000000000000000000000000000000000,
            b_knight: 0b0100001000000000000000000000000000000000000000000000000000000000,
            b_bishop: 0b0010010000000000000000000000000000000000000000000000000000000000,
            b_rook: 0b1000000100000000000000000000000000000000000000000000000000000000,
            b_queen: 0b0000100000000000000000000000000000000000000000000000000000000000,
            b_king: 0b0001000000000000000000000000000000000000000000000000000000000000,
        }
    }
}

// Returns blank bitboards
impl Default for AllBitBoards {
    fn default() -> AllBitBoards {
        AllBitBoards {
            w_pawn: 0,
            w_knight: 0,
            w_bishop: 0,
            w_rook: 0,
            w_queen: 0,
            w_king: 0,
            b_pawn: 0,
            b_knight: 0,
            b_bishop: 0,
            b_rook: 0,
            b_queen: 0,
            b_king: 0
        }
    }
}


// TODO: Refactor Apply_move
// TODO: Implement fen String
//    TODO: Implement Fen String Constructor
// TODO:
// TODO: Implement new BitBoard Exports ***returns bitboards on criteria***
//      TODO: Diagonal Pieces           (Piece)
//      TODO: Sliding Pieces            (Piece)
//      TODO: Attacked_By               (Player)  **all squares attacked by player**
//      TODO: All_Moves_Of_Piece        (Piece, Player) **All Possible Moves given Piece, Player**




pub fn left_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits >> 1},
        Player::Black => {bits << 1},
    }
}

pub fn right_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits << 1},
        Player::Black => {bits >> 1},
    }
}

pub fn up_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits << 8},
        Player::Black => {bits >> 8},
    }
}

pub fn down_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits >> 8},
        Player::Black => {bits << 8},
    }
}

pub fn safe_l_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits >> 1)  & !FILE_H},
        Player::Black => {(bits << 1)  & !FILE_A},
    }
}

pub fn safe_r_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits << 1) & ! FILE_A},
        Player::Black => {(bits >> 1) & ! FILE_H},
    }
}

pub fn safe_u_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits << 8) & !RANK_1},
        Player::Black => {(bits >> 8) & !RANK_8},
    }
}

pub fn safe_d_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits >> 8) & !RANK_8},
        Player::Black => {(bits << 8) & !RANK_1},
    }
}

pub fn left_move(bits: SQ, player:Player) -> SQ {
    match player {
        Player::White => {bits - 1},
        Player::Black => {bits + 1},
    }
}

pub fn right_move(bits: SQ, player:Player) -> SQ {
    match player {
        Player::White => {bits + 1},
        Player::Black => {bits - 1},
    }
}

pub fn up_move(bits: SQ, player:Player) -> SQ {
    match player {
        Player::White => {bits + 8},
        Player::Black => {bits - 8},
    }
}

pub fn down_move(bits: SQ, player:Player) -> SQ {
    match player {
        Player::White => {bits - 8},
        Player::Black => {bits + 8},
    }
}

pub fn REL_RANK8(player:Player) -> BitBoard {
    match player {
        Player::White => {RANK_8},
        Player::Black => {RANK_1},
    }
}

pub fn REL_RANK7(player:Player) -> BitBoard {
    match player {
        Player::White => {RANK_7},
        Player::Black => {RANK_2},
    }
}

pub fn REL_RANK5(player:Player) -> BitBoard {
    match player {
        Player::White => {RANK_5},
        Player::Black => {RANK_4},
    }
}

pub fn REL_RANK3(player:Player) -> BitBoard {
    match player {
        Player::White => {RANK_3},
        Player::Black => {RANK_6},
    }
}

pub fn left_file(player:Player) -> BitBoard {
    match player {
        Player::White => {FILE_A},
        Player::Black => {FILE_H},
    }
}

pub fn right_file(player:Player) -> BitBoard {
    match player {
        Player::White => {FILE_H},
        Player::Black => {FILE_A},
    }
}

pub fn up_left_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits << 7},
        Player::Black => {bits >> 7},
    }
}

pub fn up_right_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits << 9},
        Player::Black => {bits >> 9},
    }
}

pub fn down_left_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits >> 9},
        Player::Black => {bits << 9},
    }
}

pub fn down_right_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {bits >> 7},
        Player::Black => {bits << 7},
    }
}

pub fn safe_u_l_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits << 7) & !FILE_H & !RANK_1},
        Player::Black => {(bits >> 7) & !FILE_A & !RANK_8},
    }
}

pub fn safe_u_r_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits << 9) & !FILE_A & !RANK_1},
        Player::Black => {(bits >> 9) & !FILE_A & !RANK_1},
    }
}

pub fn safe_d_l_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits >> 9) & !FILE_H & !RANK_8},
        Player::Black => {(bits << 9) & !FILE_A & !RANK_1},
    }
}

pub fn safe_d_r_shift(bits: BitBoard, player:Player) -> BitBoard {
    match player {
        Player::White => {(bits >> 7) & !FILE_A & !RANK_8},
        Player::Black => {(bits << 7) & !FILE_H & !RANK_1},
    }
}






