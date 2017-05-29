use board::*;
use templates::Piece as Piece;
use templates::Player as Player;
use bit_twiddles::pop_count;
use piece_move::BitMove;


// TODO: Change so structs of bitboard dont have to be public
// Meaning that it stores local copies of the stuff

pub fn generate_board(fen: String) -> Result<Board, String> {
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