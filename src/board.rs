use templates::Piece as Piece;
use templates::Player as Player;
use bit_twiddles::pop_count;
use piece_move::BitMove;


pub const BLACK_SIDE: u64 = 0b1111111111111111111111111111111100000000000000000000000000000000;
pub const WHITE_SIDE: u64 = 0b0000000000000000000000000000000011111111111111111111111111111111;

pub const FILE_A: u64 = 0b0000000100000001000000010000000100000001000000010000000100000001;
pub const FILE_B: u64 = 0b0000001000000010000000100000001000000010000000100000001000000010;
pub const FILE_C: u64 = 0b0000010000000100000001000000010000000100000001000000010000000100;
pub const FILE_D: u64 = 0b0000100000001000000010000000100000001000000010000000100000001000;
pub const FILE_E: u64 = 0b0001000000010000000100000001000000010000000100000001000000010000;
pub const FILE_F: u64 = 0b0010000000100000001000000010000000100000001000000010000000100000;
pub const FILE_G: u64 = 0b0100000001000000010000000100000001000000010000000100000001000000;
pub const FILE_H: u64 = 0b1000000010000000100000001000000010000000100000001000000010000000;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;


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

// Used for determining checks
#[derive(Copy, Clone)]
pub struct LastMoveData {
    pub piece_moved: Piece,
    pub src: u8,
    pub dst: u8,
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

#[derive(Clone)]
pub struct Board {
    pub bit_boards: AllBitBoards,
    pub turn: Player,
    pub depth: u16,
   // Tracks the depth of the current board, used by Bots
    pub castling: u8,
    // 0000WWBB, left = 1 -> king side castle available, right = 1 -> queen side castle available
    pub en_passant: u8,
    // is the square of the enpassant unless equal to 2^64
    pub undo_moves: Vec<BitMove>,
    // Full list of undo-able moves
    pub ply: u8,
    // Tracks how many half-moves has been played so far
    pub last_move: Option<LastMoveData>
    // Tracks last moved played for evaluation purposes
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

// FEN
impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            bit_boards: AllBitBoards::new(),
            turn: Player::White,
            depth: 0,
            castling: 0,
            en_passant: 0,
            undo_moves: Vec::new(),
            ply: 0,
            last_move: None
        };
        board
    }

    // https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
    // "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",
    pub fn new_from_fen(fen: String) -> Result<Board, String> {
        let mut chars = fen.chars();
        let mut all_bit_boards = AllBitBoards {
            white_pawn: BitBoard { bits: 0, side: Player::White, piece: Piece::P },
            white_knight: BitBoard { bits: 0, side: Player::White, piece: Piece::N },
            white_bishop: BitBoard { bits: 0, side: Player::White, piece: Piece::B },
            white_rook: BitBoard { bits: 0, side: Player::White, piece: Piece::R },
            white_queen: BitBoard { bits: 0, side: Player::White, piece: Piece::Q },
            white_king: BitBoard { bits: 0, side: Player::White, piece: Piece::K },
            black_pawn: BitBoard { bits: 0, side: Player::Black, piece: Piece::P },
            black_knight: BitBoard { bits: 0, side: Player::Black, piece: Piece::N },
            black_bishop: BitBoard { bits: 0, side: Player::Black, piece: Piece::B },
            black_rook: BitBoard { bits: 0, side: Player::Black, piece: Piece::R },
            black_queen: BitBoard { bits: 0, side: Player::Black, piece: Piece::Q },
            black_king: BitBoard { bits: 0, side: Player::Black, piece: Piece::K },
        };
        let mut file: u64 = 0;
        let mut castle_bits: u8 = 0;
        let mut en_passant: u8 = 0;
        let mut ply: u8 = 0;
        let mut halfmove = 0;
        let mut turn: Player = Player::White;
        // [7 - 0] -> Files
        // -1      -> Side to Move
        // -2      -> Castling Ability
        // -3      -> En Passant target Square
        // -4      -> Halfmove clock
        // -5      -> FullMove clock
        let mut end_of_line: bool = false;
        let mut pos: u64 = 0; // Start at A
        while file < 13 {
            let char = match chars.next() {
                Some(x) => x,
                None => { if end_of_line { file = 13; '&'} else {
                    return Err(format!("Ran out of Chars: Line 150 {}", file).to_owned()) }
                },
            };
            match file {
                0 ... 7 => {
                    match char {
                        '/' | ' ' => {
                            file += 1;
                            pos = 0;
                        },
                        '1' => { pos += 1; },
                        '2' => { pos += 2; },
                        '3' => { pos += 3; },
                        '4' => { pos += 4; },
                        '5' => { pos += 5; },
                        '6' => { pos += 6; },
                        '7' => { pos += 7; },
                        '8' => { pos += 8; },
                        'p' => {
                            all_bit_boards.black_pawn.bits |= (1 as u64) << ((8 * (7 - file)) + pos);
                            pos += 1;
                        },
                        'b' => {
                            all_bit_boards.black_bishop.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'n' => {
                            all_bit_boards.black_knight.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'r' => {
                            all_bit_boards.black_rook.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'q' => {
                            all_bit_boards.black_queen.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'k' => {
                            all_bit_boards.black_king.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'P' => {
                            all_bit_boards.white_pawn.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'B' => {
                            all_bit_boards.white_bishop.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'N' => {
                            all_bit_boards.white_knight.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'R' => {
                            all_bit_boards.white_rook.bits |= 1 << (8 * (7 - file) + pos);;
                            pos += 1;
                        },
                        'Q' => {
                            all_bit_boards.white_queen.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'K' => {
                            all_bit_boards.white_king.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        _ => { let e = format!("FAILED CHAR AT {}", char.to_string());
                            return Err(e.to_owned()); }
                    };
                }
                8 => {
                    match char {
                        'w' => {},
                        'b' => { turn = Player::Black; },
                        ' ' => {
                            file += 1;
                            pos = 0;
                        },
                        _ => { let e = format!("Failed Matching turn: char {}", char).to_string();
                            return Err(e.to_owned()); }
                    };
                }
                9 => {
                    match char {
                        'K' => { castle_bits |= 0b1000; }
                        'Q' => { castle_bits |= 0b0100; }
                        'k' => { castle_bits |= 0b0010; }
                        'q' => { castle_bits |= 0b0001; }
                        '-' => {}
                        ' ' => {
                            file += 1;
                            pos = 0;
                        }
                        _ => {  let e = format!("Failed Matching Castling: char: {}", char).to_string();
                                return Err(e.to_owned()); }
                    };
                }
                10 => {
                    let mut ep_position = -1;
                    match pos {
                        0 => {
                            match char {
                                '-' => {}
                                ' ' => {
                                    file += 1;
                                    pos = 0
                                }
                                'a' => { ep_position = 0; }
                                'b' => { ep_position = 1; }
                                'c' => { ep_position = 2; }
                                'd' => { ep_position = 3; }
                                'e' => { ep_position = 4; }
                                'f' => { ep_position = 5; }
                                'g' => { ep_position = 6; }
                                'h' => { ep_position = 7; }
                                _ => { let e = format!("Failed Matching EP position: char {}", char).to_string();
                                    return Err(e.to_owned()); }
                            };
                            pos += 1;
                        }
                        1 => {
                            match char {
                                '-' => {}
                                ' ' => {
                                    file += 1;
                                    pos = 0
                                }
                                '3' => { ep_position += 16; }
                                '6' => { ep_position += 30; }
                                _ => { let e = format!("Failed Matching EP File: char {}", char).to_string();
                                    return Err(e.to_owned()); }
                            };
                            pos += 1;
                        }
                        _ => { match char {
                            ' ' => {
                                file += 1;
                                pos = 0
                            },
                            _ => {
                                let e = format!("Failed Matching OverallEP Count: char {}", char).to_string();
                                return Err(e.to_owned()); }
                            };
                        }
                    };
                    let en_passant = match ep_position {
                        -1 => 64,
                        e @ _ => e,
                    };
                }
                11 => {
                    match char {
                        e @ '1' | e @ '2' | e @ '3' | e @ '4' | e @ '5' | e @ '6' | e @ '7' | e @ '8' | e @ '9' | e @ '0' => {
                            if pos == 0 {
                                halfmove = e.to_string().parse::<u64>().unwrap() as u64;
                                pos += 1;
                            } else {
                                halfmove = halfmove * 10;
                                halfmove += e.to_string().parse::<u64>().unwrap() as u64;
                                pos += 1;
                            }
                        }
                        ' ' => {
                            file += 1;
                            pos = 0
                        }
                        _ => { let e = format!("Failed Matching Halfmove Counter: char {}", char).to_string();
                            return Err(e.to_owned()); }
                    };
                }
                12 => {
                    end_of_line = true;
                    match char {
                        e @ '1' | e @ '2' | e @ '3' | e @ '4' | e @ '5' | e @ '6' | e @ '7' | e @ '8' | e @ '9' | e @ '0' => {
                            if pos == 0 {
                                ply = e.to_string().parse::<u8>().unwrap() as u8;
                                pos += 1;
                            } else {
                                ply = ply * 10;
                                ply += e.to_string().parse::<u8>().unwrap() as u8;
                                pos += 1;
                            }
                        }
                        ' ' => {
                            file += 1;
                            pos = 0
                        }
                        _ => { let e = format!("Failed Matching Ply count: char {}", char).to_string();
                            return Err(e.to_owned()); }
                    };
                }
                _ => { file = 13 }
            };
        };

        ply *= 2;
        ply =  ply - 2;
        match turn {
            Player::Black => { ply += 1 }
            _ => {}
        };
        Ok(Board {
            bit_boards: all_bit_boards,
            turn: turn,
            depth: 0,
            castling: castle_bits,
            en_passant: en_passant,
            undo_moves: Vec::new(),
            ply: ply,
            last_move: None
        })
    }

    pub fn count_piece(&self, player: Player, piece: Piece) -> u8 {
        let x = match self.get_bitboard(player, piece) {
            Some(x) => x,
            None => 0,
        };
        pop_count(x)
    }

    pub fn count_pieces_player(&self, player: Player) -> u8 {
        pop_count(self.get_occupied_player(player))
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
            }
            Player::Black => {
                match piece {
                    Piece::K => Some(self.bit_boards.black_king.bits),
                    Piece::Q => Some(self.bit_boards.black_queen.bits),
                    Piece::R => Some(self.bit_boards.black_rook.bits),
                    Piece::B => Some(self.bit_boards.black_bishop.bits),
                    Piece::N => Some(self.bit_boards.black_knight.bits),
                    Piece::P => Some(self.bit_boards.black_pawn.bits),
                }
            }
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
            }
            Player::Black => {
                vector.push(self.bit_boards.black_king);
                vector.push(self.bit_boards.black_queen);
                vector.push(self.bit_boards.black_rook);
                vector.push(self.bit_boards.black_bishop);
                vector.push(self.bit_boards.black_knight);
                vector.push(self.bit_boards.black_pawn);
            }
            _ => {}
        };
        vector
    }

    // Gets all occupied Squares
    pub fn get_occupied(&self) -> u64 {
        self.bit_boards.into_iter().fold(0, |sum, x| sum ^ x)
    }

    // Returns Deep clone of current board with Past Move List
    pub fn deep_clone(&self) -> Board {
        Board {
            bit_boards: AllBitBoards::new(),
            turn: self.turn,
            depth: self.depth,
            castling: self.castling,
            en_passant: self.en_passant,
            undo_moves: self.undo_moves.clone(),
            ply: self.ply,
            last_move: self.last_move
        }
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
            last_move: match self.last_move {
                Some(x) => Some(x),
                None => None
            }
        }
    }

    // Applies the bitmove to the board;
    pub fn apply_move(&mut self, bit_move: BitMove) {
        let us: Player = self.turn;
        let them: Player = match us {
            Player::White => Player::Black,
            Player::Black => Player::White
        };
        let src: u8 = bit_move.get_src();
        let dst: u8 = bit_move.get_dest();

        let src_bit: u64 = 1 << src;
        let dst_bit: u64 = 1 << dst;

        if bit_move.is_castle() {
            // IF CASTLE MOVE
            // White: King at index: 4
            // Black: King at index: 60
            if bit_move.is_king_castle() {
                // White: Rook at index: 7
                // Black: Rook at index: 63
                match us {
                    Player::White => {
                        let rook_pos: u64 = 1 << 7 | 1 << 5;
                        let king_pos: u64 = 1 << 4 | 1 << 6;
                        self.bit_boards.white_rook.bits ^= rook_pos;
                        self.bit_boards.white_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 7, dst: 5 });
                    }
                    Player::Black => {
                        let rook_pos: u64 = 1 << 63 | 1 << 61;
                        let king_pos: u64 = 1 << 60 | 1 << 62;
                        self.bit_boards.black_rook.bits ^= rook_pos;
                        self.bit_boards.black_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 63, dst: 61 });
                    }
                }
            } else {
                // White: Rook at index: 0
                // Black: Rook at index: 56
                match us {
                    Player::White => {
                        let rook_pos: u64 = 1 << 0 | 1 << 3;
                        let king_pos: u64 = 1 << 4 | 1 << 2;
                        self.bit_boards.white_rook.bits ^= rook_pos;
                        self.bit_boards.white_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 0, dst: 3 });
                    }
                    Player::Black => {
                        let rook_pos: u64 = 1 << 56 | 1 << 59;
                        let king_pos: u64 = 1 << 60 | 1 << 58;
                        self.bit_boards.black_rook.bits ^= rook_pos;
                        self.bit_boards.black_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 56, dst: 59 });
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
                Player::White => { self.bit_boards.white_pawn.bits ^= src_bit | dst_bit; }
                Player::Black => { self.bit_boards.black_pawn.bits ^= src_bit | dst_bit; }
            }
            self.en_passant = dst;
            self.last_move = Some(LastMoveData { piece_moved: Piece::P, src: src, dst: dst });
        } else if bit_move.is_promo() {
            if bit_move.is_capture() {
                self.xor_bitboard_player_sq(them, dst_bit);
            }
            self.xor_bitboard_player_piece_sq(us, Piece::P, src_bit);
            self.xor_bitboard_player_piece_sq(us, bit_move.promo_piece(), dst_bit);
            self.last_move = None;
        } else if bit_move.is_en_passant() {
            // PAWN ENPASSENT;
            match us {
                Player::White => {
                    self.bit_boards.white_pawn.bits ^= src_bit | dst_bit;
                    self.bit_boards.black_pawn.bits ^= dst_bit >> 8;
                }
                Player::Black => {
                    self.bit_boards.black_pawn.bits ^= src_bit | dst_bit;
                    self.bit_boards.white_pawn.bits ^= dst_bit << 8;
                }
            }
            self.last_move = Some(LastMoveData { piece_moved: Piece::P, src: src, dst: dst });
        } else {
            // QUIET MOVE

            if bit_move.is_capture() { self.xor_bitboard_player_sq(them, dst_bit); }
            // check if capture, if so opponent board needs to be modified;

            // Modify own board
            let piece = self.get_piece_from_src(src_bit, us).unwrap();
            self.xor_bitboard_player_piece_sq(us, piece, src_bit | dst_bit);
            self.last_move = Some( LastMoveData { piece_moved: piece, src: src, dst: dst } );
        }
        if !bit_move.is_double_push().0 { self.en_passant = 64; }

        self.ply += 1;
        self.turn = them;
    }

    // Returns the piece at the given place
    fn get_piece_from_src(&self, src_bit: u64, player: Player) -> Option<Piece> {
        if self.get_bitboard(player, Piece::P).unwrap() & src_bit != 0 { return Some(Piece::P) };
        if self.get_bitboard(player, Piece::R).unwrap() & src_bit != 0 { return Some(Piece::R) };
        if self.get_bitboard(player, Piece::N).unwrap() & src_bit != 0 { return Some(Piece::N) };
        if self.get_bitboard(player, Piece::Q).unwrap() & src_bit != 0 { return Some(Piece::Q) };
        if self.get_bitboard(player, Piece::B).unwrap() & src_bit != 0 { return Some(Piece::B) };
        if self.get_bitboard(player, Piece::K).unwrap() & src_bit != 0 { return Some(Piece::K) };
        None
    }

    // XORs the Bitboard of (player,piece) by the input bit_board, figures out the piece & player itself
    fn xor_bitboard_sq(&mut self, square_bit: u64) {
        let player = match self.get_bitboards_player(Player::White).iter().fold(0, |sum, x| sum ^ x.bits) & square_bit {
            0 => Player::Black,
            _ => Player::White
        };
        self.xor_bitboard_player_sq(player, square_bit);
    }

    // XORs the Bitboard of (player,piece) by the input bit_board, figures out the piece itself
    fn xor_bitboard_player_sq(&mut self, player: Player, square_bit: u64) {
        let piece = self.get_piece_from_src(square_bit, player).unwrap();
        self.xor_bitboard_player_piece_sq(player, piece, square_bit);
    }

    // XORs the Bitboard of (piece, player) by the input bit_board
    fn xor_bitboard_player_piece_sq(&mut self, player: Player, piece: Piece, square_bit: u64) {
        match player {
            Player::White => {
                match piece {
                    Piece::B => self.bit_boards.white_bishop.bits ^= square_bit,
                    Piece::P => self.bit_boards.white_pawn.bits ^= square_bit,
                    Piece::R => self.bit_boards.white_rook.bits ^= square_bit,
                    Piece::N => self.bit_boards.white_knight.bits ^= square_bit,
                    Piece::K => self.bit_boards.white_king.bits ^= square_bit,
                    Piece::Q => self.bit_boards.white_queen.bits ^= square_bit,
                };
            }
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.black_bishop.bits ^= square_bit,
                    Piece::P => self.bit_boards.black_pawn.bits ^= square_bit,
                    Piece::R => self.bit_boards.black_rook.bits ^= square_bit,
                    Piece::N => self.bit_boards.black_knight.bits ^= square_bit,
                    Piece::K => self.bit_boards.black_king.bits ^= square_bit,
                    Piece::Q => self.bit_boards.black_queen.bits ^= square_bit,
                };
            }
        };
    }

    // Sets the Bitboard of piece, player to parameter bit_board
    fn modifiy_bitboard(&mut self, bit_board: u64, player: Player, piece: Piece) {
        match player {
            Player::White => {
                match piece {
                    Piece::B => self.bit_boards.white_bishop.bits = bit_board,
                    Piece::P => self.bit_boards.white_pawn.bits = bit_board,
                    Piece::R => self.bit_boards.white_rook.bits = bit_board,
                    Piece::N => self.bit_boards.white_knight.bits = bit_board,
                    Piece::K => self.bit_boards.white_king.bits = bit_board,
                    Piece::Q => self.bit_boards.white_queen.bits = bit_board,
                };
            }
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

    // Horizontally moving and Vertically moving pieves
    pub fn sliding_piece_bits(&self, player: Player) -> u64 {
        match player {
            Player::White => self.bit_boards.white_queen.bits ^ self.bit_boards.white_rook.bits,
            Player::Black => self.bit_boards.black_queen.bits ^ self.bit_boards.black_rook.bits,
        }
    }
    // Diagonal moving pieces
    pub fn diagonal_piece_bits(&self, player: Player) -> u64 {
        match player {
            Player::White => self.bit_boards.white_queen.bits ^ self.bit_boards.white_bishop.bits,
            Player::Black => self.bit_boards.black_queen.bits ^ self.bit_boards.black_bishop.bits,
        }
    }

    pub fn get_occupied_player(&self, player: Player) -> u64 {
        match player {
            Player::White => self.get_bitboards_player(Player::White).iter().fold(0, |sum, x| sum ^ x.bits),
            Player::Black => self.get_bitboards_player(Player::Black).iter().fold(0, |sum, x| sum ^ x.bits),
        }
    }



}

impl AllBitBoards {
    pub fn new() -> AllBitBoards {
        let mut bit_boards = AllBitBoards {
            white_pawn: BitBoard { bits: 0b0000000000000000000000000000000000000000000000001111111100000000, side: Player::White, piece: Piece::P },
            white_knight: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000001000010, side: Player::White, piece: Piece::N },
            white_bishop: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000100100, side: Player::White, piece: Piece::B },
            white_rook: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000010000001, side: Player::White, piece: Piece::R },
            white_queen: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000001000, side: Player::White, piece: Piece::Q },
            white_king: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000010000, side: Player::White, piece: Piece::K },
            black_pawn: BitBoard { bits: 0b0000000011111111000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::P },
            black_knight: BitBoard { bits: 0b0100001000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::N },
            black_bishop: BitBoard { bits: 0b0010010000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::B },
            black_rook: BitBoard { bits: 0b1000000100000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::R },
            black_queen: BitBoard { bits: 0b0000100000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::Q },
            black_king: BitBoard { bits: 0b0001000000000000000000000000000000000000000000000000000000000000, side: Player::Black, piece: Piece::K },
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

// TODO: Refactor Apply_move
// TODO: Implement fen String
//    TODO: Implement Fen String Constructor
// TODO:
// TODO: Implement new BitBoard Exports ***returns bitboards on criteria***
//      TODO: Diagonal Pieces           (Piece)
//      TODO: Sliding Pieces            (Piece)
//      TODO: Attacked_By               (Player)  **all squares attacked by player**
//      TODO: All_Moves_Of_Piece        (Piece, Player) **All Possible Moves given Piece, Player**


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
        let slice = &s[x * 8..(x * 8) + 8];
        print!("{}\n", slice);
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






