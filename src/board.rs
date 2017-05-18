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
    w_pawn: BitBoard,
    w_knight: BitBoard,
    w_bishop: BitBoard,
    w_rook: BitBoard,
    w_queen: BitBoard,
    w_king: BitBoard,
    b_pawn: BitBoard,
    b_knight: BitBoard,
    b_bishop: BitBoard,
    b_rook: BitBoard,
    b_queen: BitBoard,
    b_king: BitBoard,
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
        Board {
            bit_boards: AllBitBoards::new(),
            turn: Player::White,
            depth: 0,
            castling: 0,
            en_passant: 0,
            undo_moves: Vec::new(),
            ply: 0,
            last_move: None
        }
    }

    // https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
    // "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",
    pub fn new_from_fen(fen: String) -> Result<Board, String> {
        let mut chars = fen.chars();
        let mut all_bit_boards = AllBitBoards::default();
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
                            all_bit_boards.b_pawn.bits |= (1 as u64) << ((8 * (7 - file)) + pos);
                            pos += 1;
                        },
                        'b' => {
                            all_bit_boards.b_bishop.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'n' => {
                            all_bit_boards.b_knight.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'r' => {
                            all_bit_boards.b_rook.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'q' => {
                            all_bit_boards.b_queen.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'k' => {
                            all_bit_boards.b_king.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'P' => {
                            all_bit_boards.w_pawn.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'B' => {
                            all_bit_boards.w_bishop.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'N' => {
                            all_bit_boards.w_knight.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'R' => {
                            all_bit_boards.w_rook.bits |= 1 << (8 * (7 - file) + pos);;
                            pos += 1;
                        },
                        'Q' => {
                            all_bit_boards.w_queen.bits |= 1 << (8 * (7 - file) + pos);
                            pos += 1;
                        },
                        'K' => {
                            all_bit_boards.w_king.bits |= 1 << (8 * (7 - file) + pos);
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
                    let mut ep_position = 64;
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
                    en_passant = if ep_position < 64 && ep_position >= 0 {ep_position} else {64};
                }
                11 => {
                    match char {
                        e @ '1' | e @ '2' | e @ '3' | e @ '4' | e @ '5' | e @ '6' | e @ '7' | e @ '8' | e @ '9' | e @ '0' => {
                            if pos == 0 {
                                halfmove = e.to_string().parse::<u64>().unwrap() as u64;
                                pos += 1;
                            } else {
                                halfmove *= 10;
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
                                ply *= 10;
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
        ply -= 2;
        if let Player::Black = turn { ply += 1 };
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
        let x = self.get_bitboard(player, piece);
        pop_count(x)
    }

    pub fn count_pieces_player(&self, player: Player) -> u8 {
        pop_count(self.get_occupied_player(player))
    }

    // Returns Bitboard for one Piece and One Player
    pub fn get_bitboard(&self, player: Player, piece: Piece) -> u64 {
        match player {
            Player::White => {
                match piece {
                    Piece::K => (self.bit_boards.w_king.bits),
                    Piece::Q => (self.bit_boards.w_queen.bits),
                    Piece::R => (self.bit_boards.w_rook.bits),
                    Piece::B => (self.bit_boards.w_bishop.bits),
                    Piece::N => (self.bit_boards.w_knight.bits),
                    Piece::P => (self.bit_boards.w_pawn.bits),
                }
            }
            Player::Black => {
                match piece {
                    Piece::K => (self.bit_boards.b_king.bits),
                    Piece::Q => (self.bit_boards.b_queen.bits),
                    Piece::R => (self.bit_boards.b_rook.bits),
                    Piece::B => (self.bit_boards.b_bishop.bits),
                    Piece::N => (self.bit_boards.b_knight.bits),
                    Piece::P => (self.bit_boards.b_pawn.bits),
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
    pub fn get_occupied(&self) -> u64 {
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
                        self.bit_boards.w_rook.bits ^= rook_pos;
                        self.bit_boards.w_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 7, dst: 5 });
                    }
                    Player::Black => {
                        let rook_pos: u64 = 1 << 63 | 1 << 61;
                        let king_pos: u64 = 1 << 60 | 1 << 62;
                        self.bit_boards.b_rook.bits ^= rook_pos;
                        self.bit_boards.b_king.bits ^= king_pos;
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
                        self.bit_boards.w_rook.bits ^= rook_pos;
                        self.bit_boards.w_king.bits ^= king_pos;
                        self.last_move = Some(LastMoveData { piece_moved: Piece::R, src: 0, dst: 3 });
                    }
                    Player::Black => {
                        let rook_pos: u64 = 1 << 56 | 1 << 59;
                        let king_pos: u64 = 1 << 60 | 1 << 58;
                        self.bit_boards.b_rook.bits ^= rook_pos;
                        self.bit_boards.b_king.bits ^= king_pos;
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
                Player::White => { self.bit_boards.w_pawn.bits ^= src_bit | dst_bit; }
                Player::Black => { self.bit_boards.b_pawn.bits ^= src_bit | dst_bit; }
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
                    self.bit_boards.w_pawn.bits ^= src_bit | dst_bit;
                    self.bit_boards.b_pawn.bits ^= dst_bit >> 8;
                }
                Player::Black => {
                    self.bit_boards.b_pawn.bits ^= src_bit | dst_bit;
                    self.bit_boards.w_pawn.bits ^= dst_bit << 8;
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
        if self.get_bitboard(player, Piece::P) & src_bit != 0 { return Some(Piece::P) };
        if self.get_bitboard(player, Piece::R) & src_bit != 0 { return Some(Piece::R) };
        if self.get_bitboard(player, Piece::N) & src_bit != 0 { return Some(Piece::N) };
        if self.get_bitboard(player, Piece::Q) & src_bit != 0 { return Some(Piece::Q) };
        if self.get_bitboard(player, Piece::B) & src_bit != 0 { return Some(Piece::B) };
        if self.get_bitboard(player, Piece::K) & src_bit != 0 { return Some(Piece::K) };
        None
    }

    // XORs the Bitboard of (player,piece) by the input bit_board, figures out the piece & player itself
    fn xor_bitboard_sq(&mut self, square_bit: u64) {
        let player = match self.get_occupied() & square_bit {
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
                    Piece::B => self.bit_boards.w_bishop.bits ^= square_bit,
                    Piece::P => self.bit_boards.w_pawn.bits ^= square_bit,
                    Piece::R => self.bit_boards.w_rook.bits ^= square_bit,
                    Piece::N => self.bit_boards.w_knight.bits ^= square_bit,
                    Piece::K => self.bit_boards.w_king.bits ^= square_bit,
                    Piece::Q => self.bit_boards.w_queen.bits ^= square_bit,
                };
            }
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.b_bishop.bits ^= square_bit,
                    Piece::P => self.bit_boards.b_pawn.bits ^= square_bit,
                    Piece::R => self.bit_boards.b_rook.bits ^= square_bit,
                    Piece::N => self.bit_boards.b_knight.bits ^= square_bit,
                    Piece::K => self.bit_boards.b_king.bits ^= square_bit,
                    Piece::Q => self.bit_boards.b_queen.bits ^= square_bit,
                };
            }
        };
    }

    // Sets the Bitboard of piece, player to parameter bit_board
    fn modifiy_bitboard(&mut self, bit_board: u64, player: Player, piece: Piece) {
        match player {
            Player::White => {
                match piece {
                    Piece::B => self.bit_boards.w_bishop.bits = bit_board,
                    Piece::P => self.bit_boards.w_pawn.bits = bit_board,
                    Piece::R => self.bit_boards.w_rook.bits = bit_board,
                    Piece::N => self.bit_boards.w_knight.bits = bit_board,
                    Piece::K => self.bit_boards.w_king.bits = bit_board,
                    Piece::Q => self.bit_boards.w_queen.bits = bit_board,
                };
            }
            Player::Black => {
                match piece {
                    Piece::B => self.bit_boards.b_bishop.bits = bit_board,
                    Piece::P => self.bit_boards.b_pawn.bits = bit_board,
                    Piece::R => self.bit_boards.b_rook.bits = bit_board,
                    Piece::N => self.bit_boards.b_knight.bits = bit_board,
                    Piece::K => self.bit_boards.b_king.bits = bit_board,
                    Piece::Q => self.bit_boards.b_queen.bits = bit_board,
                };
            }
        };
    }

    // Horizontally moving and Vertically moving pieves
    pub fn sliding_piece_bits(&self, player: Player) -> u64 {
        match player {
            Player::White => self.bit_boards.w_queen.bits ^ self.bit_boards.w_rook.bits,
            Player::Black => self.bit_boards.b_queen.bits ^ self.bit_boards.b_rook.bits,
        }
    }
    // Diagonal moving pieces
    pub fn diagonal_piece_bits(&self, player: Player) -> u64 {
        match player {
            Player::White => self.bit_boards.w_queen.bits ^ self.bit_boards.w_bishop.bits,
            Player::Black => self.bit_boards.b_queen.bits ^ self.bit_boards.b_bishop.bits,
        }
    }

    pub fn get_occupied_player(&self, player: Player) -> u64 {
        match player {
            Player::White => self.occupied_white(),
            Player::Black => self.occupied_black(),
        }
    }

    fn occupied_white(&self) -> u64 {
        self.bit_boards.w_bishop.bits
            | self.bit_boards.w_pawn.bits
            | self.bit_boards.w_knight.bits
            | self.bit_boards.w_rook.bits
            | self.bit_boards.w_king.bits
            | self.bit_boards.w_queen.bits
    }

    fn occupied_black(&self) -> u64 {
        self.bit_boards.b_bishop.bits
            | self.bit_boards.b_pawn.bits
            | self.bit_boards.b_knight.bits
            | self.bit_boards.b_rook.bits
            | self.bit_boards.b_king.bits
            | self.bit_boards.b_queen.bits
    }



}

impl AllBitBoards {
    fn new() -> AllBitBoards {
        AllBitBoards {
            w_pawn: BitBoard { bits: 0b0000000000000000000000000000000000000000000000001111111100000000},
            w_knight: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000001000010},
            w_bishop: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000100100},
            w_rook: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000010000001},
            w_queen: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000001000},
            w_king: BitBoard { bits: 0b0000000000000000000000000000000000000000000000000000000000010000},
            b_pawn: BitBoard { bits: 0b0000000011111111000000000000000000000000000000000000000000000000},
            b_knight: BitBoard { bits: 0b0100001000000000000000000000000000000000000000000000000000000000},
            b_bishop: BitBoard { bits: 0b0010010000000000000000000000000000000000000000000000000000000000},
            b_rook: BitBoard { bits: 0b1000000100000000000000000000000000000000000000000000000000000000},
            b_queen: BitBoard { bits: 0b0000100000000000000000000000000000000000000000000000000000000000},
            b_king: BitBoard { bits: 0b0001000000000000000000000000000000000000000000000000000000000000},
        }
    }
}

// Returns blank bitboards
impl Default for AllBitBoards {
    fn default() -> AllBitBoards {
        AllBitBoards {
            w_pawn: BitBoard { bits: 0 },
            w_knight: BitBoard { bits: 0 },
            w_bishop: BitBoard { bits: 0 },
            w_rook: BitBoard { bits: 0 },
            w_queen: BitBoard { bits: 0 },
            w_king: BitBoard { bits: 0 },
            b_pawn: BitBoard { bits: 0 },
            b_knight: BitBoard { bits: 0 },
            b_bishop: BitBoard { bits: 0 },
            b_rook: BitBoard { bits: 0 },
            b_queen: BitBoard { bits: 0 },
            b_king: BitBoard { bits: 0 }
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




pub fn left_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits >> 1},
        Player::Black => {bits << 1},
    }
}

pub fn right_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits << 1},
        Player::Black => {bits >> 1},
    }
}

pub fn up_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits << 8},
        Player::Black => {bits >> 8},
    }
}

pub fn down_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits >> 8},
        Player::Black => {bits << 8},
    }
}

pub fn safe_l_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_r_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_u_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_d_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn left_move(bits: u8, player:Player) -> u8 {
    match player {
        Player::White => {bits - 1},
        Player::Black => {bits + 1},
    }
}

pub fn right_move(bits: u8, player:Player) -> u8 {
    match player {
        Player::White => {bits + 1},
        Player::Black => {bits - 1},
    }
}

pub fn up_move(bits: u8, player:Player) -> u8 {
    match player {
        Player::White => {bits + 8},
        Player::Black => {bits - 8},
    }
}

pub fn down_move(bits: u8, player:Player) -> u8 {
    match player {
        Player::White => {bits - 8},
        Player::Black => {bits + 8},
    }
}

pub fn REL_RANK8(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {RANK_8},
        Player::Black => {RANK_1},
    }
}

pub fn REL_RANK7(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {RANK_7},
        Player::Black => {RANK_2},
    }
}

pub fn REL_RANK5(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {RANK_5},
        Player::Black => {RANK_4},
    }
}

pub fn REL_RANK3(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {RANK_3},
        Player::Black => {RANK_6},
    }
}

pub fn left_file(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {FILE_A},
        Player::Black => {FILE_H},
    }
}

pub fn right_file(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {FILE_H},
        Player::Black => {FILE_A},
    }
}

pub fn up_left_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits << 7},
        Player::Black => {bits >> 7},
    }
}

pub fn up_right_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits << 9},
        Player::Black => {bits >> 9},
    }
}

pub fn down_left_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits >> 9},
        Player::Black => {bits << 9},
    }
}

pub fn down_right_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {bits >> 7},
        Player::Black => {bits << 7},
    }
}

pub fn safe_u_l_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_u_r_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_d_l_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}

pub fn safe_d_r_shift(bits: u64, player:Player) -> u64 {
    match player {
        Player::White => {},
        Player::Black => {},
    }
}






