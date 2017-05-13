
use templates::Piece as Piece;
use templates::Player as Player;
use bit_twiddles::pop_count;
use piece_move::BitMove;


pub const BLACK_SIDE: u64 = 0b1111111111111111111111111111111100000000000000000000000000000000;
pub const WHITE_SIDE: u64 = 0b0000000000000000000000000000000011111111111111111111111111111111;

pub const FILE_A : u64 = 0b0000000100000001000000010000000100000001000000010000000100000001;
pub const FILE_B : u64 = 0b0000001000000010000000100000001000000010000000100000001000000010;
pub const FILE_C : u64 = 0b0000010000000100000001000000010000000100000001000000010000000100;
pub const FILE_D : u64 = 0b0000100000001000000010000000100000001000000010000000100000001000;
pub const FILE_E : u64 = 0b0001000000010000000100000001000000010000000100000001000000010000;
pub const FILE_F : u64 = 0b0010000000100000001000000010000000100000001000000010000000100000;
pub const FILE_G : u64 = 0b0100000001000000010000000100000001000000010000000100000001000000;
pub const FILE_H : u64 = 0b1000000010000000100000001000000010000000100000001000000010000000;

pub const RANK_1 : u64 = 0x00000000000000FF;
pub const RANK_2 : u64 = 0x000000000000FF00;
pub const RANK_3 : u64 = 0x0000000000FF0000;
pub const RANK_4 : u64 = 0x00000000FF000000;
pub const RANK_5 : u64 = 0x000000FF00000000;
pub const RANK_6 : u64 = 0x0000FF0000000000;
pub const RANK_7 : u64 = 0x00FF000000000000;
pub const RANK_8 : u64 = 0xFF00000000000000;


pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const WEST: i8 = -1;
pub const EAST: i8 = 1;

pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;


#[derive(Copy, Clone)]
pub struct BitBoard {
    bits: u64,
    side: Player,
    piece: Piece
}

#[derive(Copy, Clone)]
pub struct AllBitBoards {
    white_pawn: BitBoard,
    white_knight: BitBoard,
    white_bishop: BitBoard,
    white_rook: BitBoard,
    white_queen: BitBoard,
    white_king: BitBoard,
    black_pawn: BitBoard,
    black_knight: BitBoard,
    black_bishop: BitBoard,
    black_rook: BitBoard,
    black_queen: BitBoard,
    black_king: BitBoard,
}

#[derive(Copy, Clone)]
pub struct Board {
    pub bit_boards: AllBitBoards,
    pub turn: Player,
    pub depth: u16, // Tracks how many moves has been played so far
    pub castling: u8, // 0000WWBB, left = 1 -> king side castle available, right = 1 -> queen side castle available
    pub en_passant: u8, // is the square of the enpassant unless equal to 2^64
    pub undo_moves: Vec<BitMove>,
    pub ply: u8,
}

pub struct BitBoardsIntoIterator {
    bit_boards: AllBitBoards,
    index: usize,
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


impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            bit_boards: AllBitBoards::new(),
            turn: Player::White,
            depth: 0,
            castling: 0,
            en_passant: 0,
            undo_moves: Vec::new(),
            ply: 0
        };
        board
    }

    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        let x = match self.get_bitboard(player,piece) {
            Some(x) => x,
            None    => 0,
        };
        pop_count(x)
    }

    // Returns Bitboard for one Piece and One Player
    pub fn get_bitboard(&self, player: Player, piece: Piece) -> Option<u64> {
        match player {
            Player::White => {
                match piece {
                    Piece::K => Some(self.bit_boards.white_king.bits),
                    Piece::Q => Some(self.bit_boards.white_queen.bits),
                    Piece::R => Some(self.bit_boards.white_rook.bits),
                    Piece::B => Some(self.bit_boards.white_bishop.bits),
                    Piece::N => Some(self.bit_boards.white_knight.bits),
                    Piece::P => Some(self.bit_boards.white_pawn.bits),
                }
            },
            Player::Black => {
                match piece {
                    Piece::K => Some(self.bit_boards.black_king.bits),
                    Piece::Q => Some(self.bit_boards.black_queen.bits),
                    Piece::R => Some(self.bit_boards.black_rook.bits),
                    Piece::B => Some(self.bit_boards.black_bishop.bits),
                    Piece::N => Some(self.bit_boards.black_knight.bits),
                    Piece::P => Some(self.bit_boards.black_pawn.bits),
                }
            },
            _ => None
        }
    }

    // Returns set of all bit boards for that specific player
    pub fn get_bitboards_player(&self, player: Player) -> Vec<BitBoard> {
        let mut vector = Vec::with_capacity(6);
        match player {
            Player::White => {
                vector.push(self.bit_boards.white_king);
                vector.push(self.bit_boards.white_queen);
                vector.push(self.bit_boards.white_rook);
                vector.push(self.bit_boards.white_bishop);
                vector.push(self.bit_boards.white_knight);
                vector.push(self.bit_boards.white_pawn);
            },
            Player::Black => {
                vector.push(self.bit_boards.black_king);
                vector.push(self.bit_boards.black_queen);
                vector.push(self.bit_boards.black_rook);
                vector.push(self.bit_boards.black_bishop);
                vector.push(self.bit_boards.black_knight);
                vector.push(self.bit_boards.black_pawn);
            },
            _ => {}
        };
        vector
    }

    // Gets all occupied Squares
    pub fn get_occupied(&self) -> u64 {
        self.bit_boards.into_iter().fold(0, |sum, x| sum ^ x)
    }

    pub fn deep_clone(&self) -> Board {
        Board {
            bit_boards: AllBitBoards::new(),
            turn: &self.turn,
            depth: &self.turn,
            castling: &self.castling,
            en_passant: &self.en_passant,
            undo_moves: &self.undo_moves.clone(),
            ply: &self.ply
        }
    }

    pub fn shallow_clone(&self) -> Board {
        Board {
            bit_boards: AllBitBoards::new(),
            turn: &self.turn,
            depth: &self.turn,
            castling: &self.castling,
            en_passant: &self.en_passant,
            undo_moves: Vec::new(),
            ply: &self.ply
        }
    }

    pub fn apply_move(&mut self, bit_move: BitMove) {
        let them = match self.turn {Player::White => Player::Black, Player::Black => Player::White};
        let src = bit_move.get_src();
        let dst = bit_move.get_dest();
        let src_bit = 1 << src;
        let dst_bit = 1 << dst;
        if bit_move.is_castle() {
            // White: King at index: 4
            // Black: King at index: 60
            if bit_move.is_king_castle() {
                // White: Rook at index: 7
                // Black: Rook at index: 63
                match self.turn {
                    Player::White => {
                        let rook_pos: u64 = (1<<7 | 1<<5);
                        let king_pos: u64 = (1<<4 | 1<<6);
                        self.bit_boards.white_rook = self.bit_boards.white_rook ^ rook_pos;
                        self.bit_boards.white_king = self.bit_boards.white_king ^ king_pos

                    },
                    Player::Black => {
                        let rook_pos: u64 = (1<<63 | 1<<61);
                        let king_pos: u64 = (1<<60 | 1<<62);
                        self.bit_boards.black_rook = self.bit_boards.black_rook ^ rook_pos;
                        self.bit_boards.black_king = self.bit_boards.black_king ^ king_pos;
                    }
                }
            } else {
                // White: Rook at index: 0
                // Black: Rook at index: 56
                match self.turn {
                    Player::White => {
                        let rook_pos: u64 = (1<<0 | 1<<3);
                        let king_pos: u64 = (1<<4 | 1<<2);
                        self.bit_boards.white_rook = self.bit_boards.white_rook ^ rook_pos;
                        self.bit_boards.white_king = self.bit_boards.white_king ^ king_pos
                    },
                    Player::Black => {
                        let rook_pos: u64 = (1<<56 | 1<<59);
                        let king_pos: u64 = (1<<60 | 1<<58);
                        self.bit_boards.black_rook = self.bit_boards.black_rook ^ rook_pos;
                        self.bit_boards.black_king = self.bit_boards.black_king ^ king_pos;
                    }
                }
            }
            match self.turn {
                Player::White => {
                    self.castling &= 0b11110011;
                }
                Player::Black => {
                    self.castling &= 0b11111100;
                }
            }
        } else if bit_move.is_double_push() {
            match self.turn {
                Player::White => { self.bit_boards.white_pawn ^= (src_bit | dst_bit); },
                Player::Black => { self.bit_boards.black_pawn ^= (src_bit | dst_bit); }
            }
            self.en_passant = dst;
        } else if bit_move.is_promo() {
            if bit_move.is_capture() {
                captured_piece = get_piece_from_src(dst_bit,them);
                captured_piece_board = get_bitboard(them,captured_piece);
                modifiy_bitboard(dst_bit ^ captured_piece_board, them, captured_piece);
            }
            modifiy_bitboard(get_bitboard(self.turn,Piece::P) ^ src_bit, self.turn, Piece::P);
            promoted_piece = bit_move.promo_piece();
            modifiy_bitboard(get_bitboard(self.turn,promoted_piece) ^ dst_bit, self.turn, promoted_piece);

        } else if bit_move.is_en_passant() {
            match self.turn {
                Player::White => {
                    self.bit_boards.white_pawn ^= (src_bit | dst_bit);
                    self.bit_boards.black_pawn ^= dst_bit >> 8;
                },
                Player::Black => {
                    self.bit_boards.black_pawn ^= (src_bit | dst_bit);
                    self.bit_boards.white_pawn ^= dst_bit << 8;
                }
            }
        } else {
            let piece = get_piece_from_src(src_bit, &self.turn);
            if bit_move.is_capture() {
                captured_piece = get_piece_from_src(dst_bit,them);
                captured_piece_board = get_bitboard(them,captured_piece);
                modifiy_bitboard(dst_bit ^ captured_piece_board, them, captured_piece);
            }
            modifiy_bitboard(get_bitboard(self.turn,piece) ^ (src_bit | dst_bit), self.turn, piece);
        }
        if !bit_move.is_double_push() { self.en_passant = 64; }
        self.ply += 1;
        self.turn += 1;
        self.turn = them;
    }

    fn get_piece_from_src(&self, src_bit: u64, player: Player) -> Piece {
        if self.get_bitboard(player, Piece::P) & src_bit != 0 { return Piece::P};
        if self.get_bitboard(player, Piece::R) & src_bit != 0 { return Piece::R};
        if self.get_bitboard(player, Piece::N) & src_bit != 0 { return Piece::N};
        if self.get_bitboard(player, Piece::Q) & src_bit != 0 { return Piece::Q};
        if self.get_bitboard(player, Piece::B) & src_bit != 0 { return Piece::B};
        Piece::K
    }

    fn modifiy_bitboard(&mut self, bit_board: u64, player: Player, piece: Piece) {
        match player {
            Player::White  => {
                match piece {
                    Piece::B => self.bit_boards.white_bishop.bits = bit_board,
                    Piece::P => self.bit_boards.white_pawn.bits = bit_board,
                    Piece::R => self.bit_boards.white_rook.bits = bit_board,
                    Piece::N => self.bit_boards.white_knight.bits = bit_board,
                    Piece::K => self.bit_boards.white_king.bits = bit_board,
                    Piece::Q => self.bit_boards.white_queen.bits = bit_board,
                };
            },
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.black_bishop.bits = bit_board,
                    Piece::P => self.bit_boards.black_pawn.bits = bit_board,
                    Piece::R => self.bit_boards.black_rook.bits = bit_board,
                    Piece::N => self.bit_boards.black_knight.bits = bit_board,
                    Piece::K => self.bit_boards.black_king.bits = bit_board,
                    Piece::Q => self.bit_boards.black_queen.bits = bit_board,
                };
            }
        };
    }


}

impl AllBitBoards {
    pub fn new() -> AllBitBoards {
        let mut bit_boards = AllBitBoards {
            white_pawn:     BitBoard {bits: 0b0000000000000000000000000000000000000000000000001111111100000000, side: Player::White, piece: Piece::P},
            white_knight:   BitBoard {bits: 0b0000000000000000000000000000000000000000000000000000000001000010, side: Player::White, piece: Piece::N},
            white_bishop:   BitBoard {bits: 0b0000000000000000000000000000000000000000000000000000000000100100, side: Player::White, piece: Piece::N},
            white_rook:     BitBoard {bits: 0b0000000000000000000000000000000000000000000000000000000010000001, side: Player::White, piece: Piece::R},
            white_queen:    BitBoard {bits: 0b0000000000000000000000000000000000000000000000000000000000001000, side: Player::White, piece: Piece::Q},
            white_king:     BitBoard {bits: 0b0000000000000000000000000000000000000000000000000000000000010000, side: Player::White, piece: Piece::K},
            black_pawn:     BitBoard {bits: 0b0000000011111111000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::P},
            black_knight:   BitBoard {bits: 0b0100001000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::N},
            black_bishop:   BitBoard {bits: 0b0010010000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::B},
            black_rook:     BitBoard {bits: 0b1000000100000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::R},
            black_queen:    BitBoard {bits: 0b0000100000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::Q},
            black_king:     BitBoard {bits: 0b0001000000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::K},
        };
        bit_boards
    }
}

impl IntoIterator for AllBitBoards {
    type Item = u64;
    type IntoIter = BitBoardsIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardsIntoIterator { bit_boards: self, index: 0 }
    }
}

impl Iterator for BitBoardsIntoIterator {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let result = match self.index {
            0 => Some(self.bit_boards.white_pawn.bits),
            1 => Some(self.bit_boards.white_knight.bits),
            2 => Some(self.bit_boards.white_bishop.bits),
            3 => Some(self.bit_boards.white_rook.bits),
            4 => Some(self.bit_boards.white_queen.bits),
            5 => Some(self.bit_boards.white_king.bits),
            6 => Some(self.bit_boards.black_pawn.bits),
            7 => Some(self.bit_boards.black_knight.bits),
            8 => Some(self.bit_boards.black_bishop.bits),
            9 => Some(self.bit_boards.black_rook.bits),
            10 => Some(self.bit_boards.black_queen.bits),
            11 => Some(self.bit_boards.black_king.bits),
            _ => return None,
        };
        self.index += 1;
        result
    }
}


pub fn main() {
    let board = Board::new();
    //    print(board);
//    print!("{}", check_board(&board));
    print!("{}", board.count_piece(Player::White, Piece::P));

}

pub fn print(board: Board) {
    let bit_board = board.bit_boards;
    for i in bit_board.into_iter() {
        print_bitboard(i);
    }
    //    let xor = bit_board.into_iter().fold(0, |sum, x| sum ^ x);
    //    println!("{}",format_u64(xor));
}

pub fn print_bitboard(input: u64) {
    let s = format_u64(input);
    for x in 0..8 {
        let slice = &s[x*8..(x*8)+8];
        print!("{}\n",slice);
    }
    println!();
}

fn format_u64(input: u64) -> String {
    let mut s = String::with_capacity(64);
    let strin = format!("{:b}", input);
    let mut i = strin.len();
    while i < 64 {
        s.push_str("0");
        i += 1;
    }
    s.push_str(&strin);
    s
}






