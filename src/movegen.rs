use templates::{SQ,Piece,Player,to_SQ};
use board::*;
use piece_move::{MoveFlag,BitMove,PreMoveInfo};
use std;
use std::num::Wrapping;

static index64: &'static[u8] = &[
    0,  1, 48,  2, 57, 49, 28,  3,
    61, 58, 50, 42, 38, 29, 17,  4,
    62, 55, 59, 36, 53, 51, 43, 22,
    45, 39, 33, 30, 24, 18, 12,  5,
    63, 47, 56, 27, 60, 41, 37, 16,
    54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10,
    25, 14, 19,  9, 13,  8,  7,  6
];

pub fn get_pseudo_moves(board: &Board, player: Player) -> Vec<PreMoveInfo> {
    let mut vec = Vec::with_capacity(40);
    get_pawn_moves(&board, player, & mut vec);
    vec
}

pub fn get_pawn_moves(board: &Board, player: Player, list: &mut Vec<PreMoveInfo>) {
    let THEM: Player = match player {Player::White => Player::Black, Player::Black => Player::White};
    let TRANK8BB: u64 = match player {Player::White => RANK_8, Player::Black => RANK_1};
    let TRANK7BB: u64 = match player {Player::White => RANK_7, Player::Black => RANK_2};
    let TRANK3BB: u64 = match player {Player::White => RANK_8, Player::Black => RANK_1};
    let UP: i8 = match player {
        Player::White => NORTH,
        Player::Black => SOUTH
        };
    let RIGHT: i8 = match player {Player::White => NORTH_EAST, Player::Black => SOUTH_WEST};
    let LEFT: i8 = match player {Player::White => NORTH_WEST, Player::Black => SOUTH_EAST};

    let pawn_bits = board.get_bitboard(player, Piece::P);
    let occupied = board.get_occupied();

    // get single pushes
    let single_push: u64 = (pawn_bits.unwrap() << UP) & !occupied;
    println!("{:b}", single_push);
    let double_push: u64 = ((single_push & TRANK3BB) << UP) & !occupied;

    let mut single_push_list = Vec::new();
    bit_scan_forward_list(single_push, &mut single_push_list);
    while single_push_list.len() > 0 {
        let dest = single_push_list.pop().unwrap();
        let sorc = dest >> UP;
        list.push(PreMoveInfo {src: to_SQ(sorc), dst: to_SQ(dest), flags: MoveFlag::QuietMove  });
    }

}


pub fn bit_scan_forward_list(input_bits: u64, list: &mut Vec<u8>) {
//    println!("{:b}", bits);
    let mut bits = input_bits;
    let mut i = 0;
    while bits != 0 && i < 30 {
        let pos = bit_scan_forward(bits);
        println!("{:b}", pos);
        println!("{:?}", pos);
        list.push(pos);

        let pos = !((1u64).checked_shl(pos as u32).unwrap());
        bits = bits & (!(pos) as u64);
//        bits = bits & !(1 << (pos));
        println!("{:b}", !pos);
        println!("{:b}", bits);
        i += 1;
    }
}

pub fn bit_scan_forward(bits: u64) -> u8 {
    const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;
    let xor_bits = bits & (!bits + 1);
    index64[( xor_bits.saturating_mul(DEBRUIJN64) >> 58) as usize]
}

pub fn debug_bit_scan(bits: u64) -> u8 {
    const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;
    print_bits(bits);
    print_bits( (bits-1));
    print_bits( bits ^ (bits-1));
    print_bits(((bits ^ (bits-1)) * DEBRUIJN64));
    print_bits((((bits ^ (bits-1)) * DEBRUIJN64) >> 58));
    print_bits(index64[(((bits ^ (bits-1)) * DEBRUIJN64) >> 58) as usize] as u64);

    index64[(((bits ^ (bits-1)) * DEBRUIJN64) >> 58) as usize] as u8
}

fn print_bits(bits: u64) {
    let strin = format!("{:b}", bits);
    println!("{:?}", strin);
}



