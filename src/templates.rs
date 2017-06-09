use bit_twiddles;

#[derive(Copy, Clone)]
pub enum Player {
    White,
    Black,
}

#[derive(Copy, Clone)]
pub struct WhitePlayer;

#[derive(Copy, Clone)]
pub struct BlackPlayer;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    K = 6,
    Q = 5,
    R = 4,
    B = 3,
    N = 2,
    P = 1,
}

pub type BitBoard = u64;
pub type SQ = u8;

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


pub const FILE_BB: [u64; 8] = [FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H];
pub const RANK_BB: [u64; 8] = [RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8];



pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const WEST: i8 = -1;
pub const EAST: i8 = 1;

pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;

#[inline]
pub fn other_player(p: Player) -> Player {
    match p {
        Player::White => Player::Black,
        Player::Black => Player::White,
    }
}

// For whatever rank the bit is in, gets the whole bitboard
#[inline]
pub fn rank_bb(s: SQ) -> BitBoard {
    RANK_BB[rank_of_sq(s) as usize]
}

#[inline]
pub fn rank_of_sq(s: SQ) -> u8 {
    (s >> 3) as u8
}

#[inline]
pub fn file_bb(s: SQ) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

#[inline]
pub fn file_of_sq(s: SQ) -> u8 {
    s & 0b00000111
}

// Assumes only one bit!
#[inline]
pub fn bb_to_sq(b: BitBoard) -> SQ {
    debug_assert_eq!(bit_twiddles::popcount64(b),1);
    bit_twiddles::bit_scan_forward(b)
}

#[inline]
pub fn sq_to_to(s: SQ) -> BitBoard {
    assert!(s < 64);
    (1 as u64) << s
}

#[inline]
pub fn sq_is_okay(s: SQ) -> bool {
    s < 64
}
