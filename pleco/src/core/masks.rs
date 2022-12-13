//! Masks and various other constants.

use super::sq::SQ;

/// The total number of players on a chessboard.
pub const PLAYER_CNT: usize = 2;
/// The total number of types of pieces on a chessboard.
pub const PIECE_TYPE_CNT: usize = 8;
/// The total number of types of pieces & player combinations on a chessboard.
pub const PIECE_CNT: usize = 16;
/// The total number of squares on a chessboard.
pub const SQ_CNT: usize = 64;
/// The total number of files on a chessboard.
pub const FILE_CNT: usize = 8;
/// The total number of ranks on a chessboard.
pub const RANK_CNT: usize = 8;

/// The total number of game phases, being middle and end game
pub const PHASE_CNT: usize = 2;
/// The total number of types of castling a player can perform: king side and
/// queen side.
pub const CASTLING_SIDES: usize = 2;
/// The total number of types of castling rights available for a chessboard,
/// concerning a singular player. eg, a player either has 1) both K side and
/// Q side castling possible, 2) K side castling possible, 3) Q side castling
/// possible, or 4) no castling possible.
pub const TOTAL_CASTLING_CNT: usize = CASTLING_SIDES * CASTLING_SIDES;
/// Total number of castling rights for both players.
pub const ALL_CASTLING_RIGHTS: usize = TOTAL_CASTLING_CNT * TOTAL_CASTLING_CNT;

/// Bit representation of the black player's side of the board.
pub const BLACK_SIDE: u64 =
    0b11111111_11111111_11111111_11111111_00000000_00000000_00000000_00000000;
/// Bit representation of the White player's side of the board.
pub const WHITE_SIDE: u64 =
    0b00000000_00000000_00000000_00000000_11111111_11111111_11111111_11111111;

/// Bit representation of file A.
pub const FILE_A: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;
/// Bit representation of file B.
pub const FILE_B: u64 = 0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
/// Bit representation of file C.
pub const FILE_C: u64 = 0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
/// Bit representation of file D.
pub const FILE_D: u64 = 0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
/// Bit representation of file E.
pub const FILE_E: u64 = 0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
/// Bit representation of file F.
pub const FILE_F: u64 = 0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000;
/// Bit representation of file H.
pub const FILE_G: u64 = 0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
/// Bit representation of file G.
pub const FILE_H: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;

/// Bit representation of rank 1.
pub const RANK_1: u64 = 0x0000_0000_0000_00FF;
/// Bit representation of rank 2.
pub const RANK_2: u64 = 0x0000_0000_0000_FF00;
/// Bit representation of rank 3.
pub const RANK_3: u64 = 0x0000_0000_00FF_0000;
/// Bit representation of rank 4.
pub const RANK_4: u64 = 0x0000_0000_FF00_0000;
/// Bit representation of rank 5.
pub const RANK_5: u64 = 0x0000_00FF_0000_0000;
/// Bit representation of rank 6.
pub const RANK_6: u64 = 0x0000_FF00_0000_0000;
/// Bit representation of rank 7.
pub const RANK_7: u64 = 0x00FF_0000_0000_0000;
/// Bit representation of rank 8.
pub const RANK_8: u64 = 0xFF00_0000_0000_0000;

/// Bit representation of rank dark squares.
pub const DARK_SQUARES: u64 = 0xAA55AA55AA55AA55;
/// Bit representation of all light squares.
pub const LIGHT_SQUARES: u64 = !DARK_SQUARES;

/// Array of all files and their corresponding bits, indexed from
/// file A to file H.
pub static FILE_BB: [u64; FILE_CNT] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

/// Array of all ranks and their corresponding bits, indexed from
/// rank 1 to rank 8.
pub static RANK_BB: [u64; RANK_CNT] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

/// Direction of going north on a chessboard.
pub const NORTH: i8 = 8;
/// Direction of going south on a chessboard.
pub const SOUTH: i8 = -8;
/// Direction of going west on a chessboard.
pub const WEST: i8 = -1;
/// Direction of going east on a chessboard.
pub const EAST: i8 = 1;

/// Direction of going northeast on a chessboard.
pub const NORTH_EAST: i8 = 9;
/// Direction of going northwest on a chessboard.
pub const NORTH_WEST: i8 = 7;
/// Direction of going southeast on a chessboard.
pub const SOUTH_EAST: i8 = -7;
/// Direction of going southwest on a chessboard.
pub const SOUTH_WEST: i8 = -9;

/// Array for starting occupancy boards for both players.
pub static START_OCC_BOARDS: [u64; PLAYER_CNT] = [START_WHITE_OCC, START_BLACK_OCC];

/// Bits for starting occupancy boards for a white pawn.
pub const START_W_PAWN: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
/// Bits for starting occupancy boards for a white knight.
pub const START_W_KNIGHT: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010;
/// Array for starting occupancy boards for a white bishop.
pub const START_W_BISHOP: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100;
/// Bits for starting occupancy boards for a white rook.
pub const START_W_ROOK: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001;
/// Bits for starting occupancy boards for a white queen.
pub const START_W_QUEEN: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000;
/// Bits for starting occupancy boards for a white king.
pub const START_W_KING: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000;

/// Bits for starting occupancy boards for a black pawn.
pub const START_B_PAWN: u64 =
    0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for starting occupancy boards for a black knight.
pub const START_B_KNIGHT: u64 =
    0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for starting occupancy boards for a black bishop.
pub const START_B_BISHOP: u64 =
    0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for starting occupancy boards for a black rook.
pub const START_B_ROOK: u64 =
    0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for starting occupancy boards for a black queen.
pub const START_B_QUEEN: u64 =
    0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for starting occupancy boards for a black king.
pub const START_B_KING: u64 =
    0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

/// Bits for the starting occupancy board for the white player.
pub const START_WHITE_OCC: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_11111111;
/// Bits for the starting occupancy board for the black player.
pub const START_BLACK_OCC: u64 =
    0b11111111_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
/// Bits for the starting occupancy board for both players.
pub const START_OCC_ALL: u64 = START_BLACK_OCC | START_WHITE_OCC;

/// Starting square number of the white king.
pub const WHITE_KING_START: u8 = 4;
/// Starting square number of the black king.
pub const BLACK_KING_START: u8 = 60;

/// Starting square number of the black king-side rook.
pub const ROOK_BLACK_KSIDE_START: u8 = 63;
/// Starting square number of the black queen-side rook.
pub const ROOK_BLACK_QSIDE_START: u8 = 56;
/// Starting square number of the white king-side rook.
pub const ROOK_WHITE_KSIDE_START: u8 = 7;
/// Starting square number of the white queen-side rook.
pub const ROOK_WHITE_QSIDE_START: u8 = 0;

/// Castling right bit representing the white king-side castle is still possible.
pub const C_WHITE_K_MASK: u8 = 0b0000_1000;
/// Castling right bit representing the white queen-side castle is still possible.
pub const C_WHITE_Q_MASK: u8 = 0b0000_0100;
/// Castling right bit representing the black king-side castle is still possible.
pub const C_BLACK_K_MASK: u8 = 0b0000_0010;
/// Castling right bit representing the black queen-side castle is still possible.
pub const C_BLACK_Q_MASK: u8 = 0b0000_0001;

/// Array containing all the starting rook positions for each side, for each player.
pub static CASTLING_ROOK_START: [[u8; CASTLING_SIDES]; PLAYER_CNT] = [
    [ROOK_WHITE_KSIDE_START, ROOK_WHITE_QSIDE_START],
    [ROOK_BLACK_KSIDE_START, ROOK_BLACK_QSIDE_START],
];

/// Bits representing the castling path for a white king-side castle.
pub const CASTLING_PATH_WHITE_K_SIDE: u64 = 1_u64 << SQ::F1.0 as u32 | 1_u64 << SQ::G1.0 as u32;
/// Bits representing the castling path for a white queen-side castle.
pub const CASTLING_PATH_WHITE_Q_SIDE: u64 =
    1_u64 << SQ::B1.0 as u32 | 1_u64 << SQ::C1.0 as u32 | 1_u64 << SQ::D1.0 as u32;

/// Bits representing the castling path for a black king-side castle.
pub const CASTLING_PATH_BLACK_K_SIDE: u64 = 1_u64 << SQ::F8.0 as u32 | 1_u64 << SQ::G8.0 as u32;
/// Bits representing the castling path for a black queen-side castle.
pub const CASTLING_PATH_BLACK_Q_SIDE: u64 =
    1_u64 << SQ::B8.0 as u32 | 1_u64 << SQ::C8.0 as u32 | 1_u64 << SQ::D8.0 as u32;

/// Array for the bits representing the castling path for a white castle, indexed
/// per the side available (king-side, queen-side).
pub static CASTLING_PATH_WHITE: [u64; CASTLING_SIDES] =
    [CASTLING_PATH_WHITE_K_SIDE, CASTLING_PATH_WHITE_Q_SIDE];

/// Array for the bits representing the castling path for a white castle, indexed
/// per the side available (king-side, queen-side).
pub static CASTLING_PATH_BLACK: [u64; CASTLING_SIDES] =
    [CASTLING_PATH_BLACK_K_SIDE, CASTLING_PATH_BLACK_Q_SIDE];

/// Array for the bits representing the castling path for castle, indexed
/// per the side available (king-side, queen-side), as well as indexed per player.
pub static CASTLING_PATH: [[u64; CASTLING_SIDES]; PLAYER_CNT] = [
    [CASTLING_PATH_WHITE_K_SIDE, CASTLING_PATH_WHITE_Q_SIDE],
    [CASTLING_PATH_BLACK_K_SIDE, CASTLING_PATH_BLACK_Q_SIDE],
];

/// Display order for a squares. Used for printing, and for answering the question
/// of which square to print first.
pub static SQ_DISPLAY_ORDER: [u8; SQ_CNT] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];

/// Array mapping a square index to a string representation.
///
/// # Examples
///
/// ```
/// use pleco::core::masks::SQ_DISPLAY;
///
/// assert_eq!(SQ_DISPLAY[0], "a1");
/// assert_eq!(SQ_DISPLAY[1], "b1");
/// assert_eq!(SQ_DISPLAY[8], "a2");
/// ```
pub static SQ_DISPLAY: [&str; SQ_CNT] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

/// Characters for each combination of player and piece.
///
/// White pieces are displayed as Uppercase letters, while black pieces are lowercase.
pub static PIECE_DISPLAYS: [[char; PIECE_TYPE_CNT]; PLAYER_CNT] = [
    ['_', 'P', 'N', 'B', 'R', 'Q', 'K', '*'],
    ['_', 'p', 'n', 'b', 'r', 'q', 'k', '*'],
];

/// Characters for each file, index from file A to file H.
/// # Examples
///
/// ```
/// use pleco::core::masks::FILE_DISPLAYS;
///
/// assert_eq!(FILE_DISPLAYS[0], 'a');
/// assert_eq!(FILE_DISPLAYS[1], 'b');
/// assert_eq!(FILE_DISPLAYS[7], 'h');
/// ```
pub static FILE_DISPLAYS: [char; FILE_CNT] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

/// Characters for each rank, index from rank 1 to rank 8.
///
/// # Examples
///
/// ```
/// use pleco::core::masks::RANK_DISPLAYS;
///
/// assert_eq!(RANK_DISPLAYS[0], '1');
/// assert_eq!(RANK_DISPLAYS[1], '2');
/// assert_eq!(RANK_DISPLAYS[7], '8');
/// ```
pub static RANK_DISPLAYS: [char; FILE_CNT] = ['1', '2', '3', '4', '5', '6', '7', '8'];
