use templates::{SQ, Piece, Player, to_SQ};
use board::*;
use piece_move::{MoveFlag, BitMove, PreMoveInfo};
use std;
use std::num::Wrapping;
use bit_twiddles::{pop_count, bit_scan_forward};

static index64: &'static [u8] = &[
    0, 1, 48, 2, 57, 49, 28, 3,
    61, 58, 50, 42, 38, 29, 17, 4,
    62, 55, 59, 36, 53, 51, 43, 22,
    45, 39, 33, 30, 24, 18, 12, 5,
    63, 47, 56, 27, 60, 41, 37, 16,
    54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10,
    25, 14, 19, 9, 13, 8, 7, 6
];


pub fn get_pseudo_moves(board: &Board, player: Player) -> Vec<PreMoveInfo> {
    let mut vec = Vec::with_capacity(40);
    get_pawn_moves(&board, player, &mut vec);
    vec
}

pub fn in_check(board: &Board) -> bool {
    let turn = board.turn;

    option = board.last_move;
    if option.unwrap() == None { return false; }


    let last_move_info: LastMoveData = option.unwrap();
    let piece_moved = last_move_info.piece_moved;
    let src = last_move_info.src;
    let dst = last_move_info.dst;
    let king_pos = board.get_bitboard(turn, Piece::K);



    true
}

pub fn get_pawn_moves(board: &Board, player: Player, list: &mut Vec<PreMoveInfo>) {
    let THEM: Player = match player {
        Player::White => Player::Black,
        Player::Black => Player::White
    };
    let TRANK8BB: u64 = match player {
        Player::White => RANK_8,
        Player::Black => RANK_1
    };
    let TRANK7BB: u64 = match player {
        Player::White => RANK_7,
        Player::Black => RANK_2
    };
    let TRANK3BB: u64 = match player {
        Player::White => RANK_3,
        Player::Black => RANK_6
    };
    let UP: i8 = match player {
        Player::White => NORTH,
        Player::Black => SOUTH
    };
    let RIGHT: i8 = match player {
        Player::White => NORTH_EAST,
        Player::Black => SOUTH_WEST
    };
    let LEFT: i8 = match player {
        Player::White => NORTH_WEST,
        Player::Black => SOUTH_EAST
    };

    let pawn_bits = board.get_bitboard(player, Piece::P).unwrap();
    let occupied = board.get_occupied();

    // get single and double pushes
    let single_push: u64 = (pawn_bits << UP) & !occupied;
    let double_push: u64 = ((single_push & TRANK3BB) << UP) & !occupied;

    // Single Moves
    let mut single_push_list = Vec::new();
    bit_scan_forward_list(single_push, &mut single_push_list);
    while single_push_list.len() > 0 {
        let dest = single_push_list.pop().unwrap();
        let sorc = match player {
            Player::White => dest - 8,
            Player::Black => dest + 8,
        };
        list.push(PreMoveInfo { src: to_SQ(sorc), dst: to_SQ(dest), flags: MoveFlag::QuietMove });
    }

    let mut double_push_list = Vec::new();
    bit_scan_forward_list(double_push, &mut double_push_list);

    // Double Moves
    while double_push_list.len() > 0 {
        let dest = double_push_list.pop().unwrap();
        let sorc = match player {
            Player::White => dest - 8,
            Player::Black => dest + 8,
        };
        list.push(PreMoveInfo { src: to_SQ(sorc), dst: to_SQ(dest), flags: MoveFlag::DoublePawnPush });
    }

    // TODO: Implement Captures

    // TODO: Implement
}


fn get_rank_mask(bit: u64) -> u64  {
    match bit_scan_forward(bit) / 8 {
        0 => board::RANK_1,
        1 => board::RANK_2,
        2 => board::RANK_3,
        3 => board::RANK_4,
        4 => board::RANK_5,
        5 => board::RANK_6,
        6 => board::RANK_7,
        7 => board::RANK_8,
    }
}

fn get_file_mask(bit: u64) -> u64  {
    match bit_scan_forward(bit) / 8 {
        0 => board::FILE_A,
        1 => board::FILE_B,
        2 => board::FILE_C,
        3 => board::FILE_D,
        4 => board::FILE_E,
        5 => board::FILE_F,
        6 => board::FILE_G,
        7 => board::FILE_H,
    }
}


pub fn bit_scan_forward_list(input_bits: u64, list: &mut Vec<u8>) {
    let mut bits = input_bits;
    while bits != 0 {
        let pos = bit_scan_forward(bits);
        list.push(pos);
        let pos = (1u64).checked_shl(pos as u32).unwrap();
        bits = bits & (!(pos) as u64);
    }
}

// TODO: Implement Knight Attacks
// TODO: Implement King Attacks
// TODO: Implement Diagonal Attacks
// TODO: Implement Sliding Attacks
// TODO: Implement Knight Attacks

// TODO: Implement Move Checker Attacks


