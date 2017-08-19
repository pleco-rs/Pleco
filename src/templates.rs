use bit_twiddles;
use std::mem;
use std::fmt;

/// Enum to represent the Players White & Black.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if self == &Player::White {
                "White"
            } else {
                "Black"
            }
        )
    }
}


pub const ALL_PLAYERS: [Player; 2] = [Player::White, Player::Black];

pub const PLAYER_CNT: usize = 2;
pub const PIECE_CNT: usize = 6;
pub const SQ_CNT: usize = 64;
pub const FILE_CNT: usize = 8;
pub const RANK_CNT: usize = 8;
pub const TOTAL_CASTLING_CNT: usize = 4;
pub const CASTLING_SIDES: usize = 2;

/// Publically available move-generation types.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GenTypes {
    All,
    Captures,
    Quiets,
    QuietChecks,
}

/// Enum for all the possible Pieces.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    K = 5,
    Q = 4,
    R = 3,
    B = 2,
    N = 1,
    P = 0,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Piece::P => "Pawn",
            Piece::N => "Knight",
            Piece::B => "Bishop",
            Piece::R => "Rook",
            Piece::Q => "Queen",
            Piece::K => "King"
        };
        f.pad(s)
    }
}

/// Enum for the Files of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

/// Enum for the Ranks of a Chessboard.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}


pub const ALL_FILES: [File; FILE_CNT] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];
pub const ALL_RANKS: [Rank; RANK_CNT] = [
    Rank::R1,
    Rank::R2,
    Rank::R3,
    Rank::R4,
    Rank::R5,
    Rank::R6,
    Rank::R7,
    Rank::R8,
];

pub const ALL_PIECES: [Piece; PIECE_CNT] =
    [Piece::P, Piece::N, Piece::B, Piece::R, Piece::Q, Piece::K];

/// BitBoard is a u64, where the bits of each index represent a square.
pub type BitBoard = u64;

/// Alias for a certain square number.
pub type SQ = u8;

pub const NO_SQ: SQ = 64;

pub const BLACK_SIDE: u64 =
    0b11111111_11111111_11111111_11111111_00000000_00000000_00000000_00000000;
pub const WHITE_SIDE: u64 =
    0b00000000_00000000_00000000_00000000_11111111_11111111_11111111_11111111;

pub const FILE_A: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;
pub const FILE_B: u64 = 0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
pub const FILE_C: u64 = 0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
pub const FILE_D: u64 = 0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
pub const FILE_E: u64 = 0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
pub const FILE_F: u64 = 0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000;
pub const FILE_G: u64 = 0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
pub const FILE_H: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;

pub const RANK_1: u64 = 0x0000_0000_0000_00FF;
pub const RANK_2: u64 = 0x0000_0000_0000_FF00;
pub const RANK_3: u64 = 0x0000_0000_00FF_0000;
pub const RANK_4: u64 = 0x0000_0000_FF00_0000;
pub const RANK_5: u64 = 0x0000_00FF_0000_0000;
pub const RANK_6: u64 = 0x0000_FF00_0000_0000;
pub const RANK_7: u64 = 0x00FF_0000_0000_0000;
pub const RANK_8: u64 = 0xFF00_0000_0000_0000;


pub const FILE_BB: [u64; 8] = [
    FILE_A,
    FILE_B,
    FILE_C,
    FILE_D,
    FILE_E,
    FILE_F,
    FILE_G,
    FILE_H,
];
pub const RANK_BB: [u64; 8] = [
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8,
];


pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const WEST: i8 = -1;
pub const EAST: i8 = 1;

pub const NORTH_EAST: i8 = 9;
pub const NORTH_WEST: i8 = 7;
pub const SOUTH_EAST: i8 = -7;
pub const SOUTH_WEST: i8 = -9;

pub const START_W_PAWN: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
pub const START_W_KNIGHT: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010;
pub const START_W_BISHOP: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100;
pub const START_W_ROOK: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001;
pub const START_W_QUEEN: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000;
pub const START_W_KING: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000;

pub const START_B_PAWN: BitBoard =
    0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_B_KNIGHT: BitBoard =
    0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_B_BISHOP: BitBoard =
    0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_B_ROOK: BitBoard =
    0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_B_QUEEN: BitBoard =
    0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_B_KING: BitBoard =
    0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

pub const START_WHITE_OCC: BitBoard =
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_11111111;
pub const START_BLACK_OCC: BitBoard =
    0b11111111_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
pub const START_OCC_ALL: BitBoard = START_BLACK_OCC | START_WHITE_OCC;

/// Types of Castling available
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CastleType {
    KingSide = 0,
    QueenSide = 1,
}

#[repr(u8)]
/// Squares available to play
pub enum Square {
    A1 = 0,  B1, C1, D1, E1, F1, G1, H1,
    A2 = 8,  B2, C2, D2, E2, F2, G2, H2,
    A3 = 16, B3, C3, D3, E3, F3, G3, H3,
    A4 = 24, B4, C4, D4, E4, F4, G4, H4,
    A5 = 32, B5, C5, D5, E5, F5, G5, H5,
    A6 = 40, B6, C6, D6, E6, F6, G6, H6,
    A7 = 48, B7, C7, D7, E7, F7, G7, H7,
    A8 = 56, B8, C8, D8, E8, F8, G8, H8,
}

pub const ROOK_BLACK_KSIDE_START: SQ = 63;
pub const ROOK_BLACK_QSIDE_START: SQ = 56;
pub const ROOK_WHITE_KSIDE_START: SQ = 7;
pub const ROOK_WHITE_QSIDE_START: SQ = 0;

pub const CASTLING_ROOK_START: [[SQ; CASTLING_SIDES]; PLAYER_CNT] =
    [
        [ROOK_WHITE_KSIDE_START, ROOK_WHITE_QSIDE_START],
        [ROOK_BLACK_KSIDE_START, ROOK_BLACK_QSIDE_START],
    ];

pub const CASTLING_PATH_WHITE_K_SIDE: u64 = (1 as u64) << (Square::F1 as u32) |
    (1 as u64) << (Square::G1 as u32);
pub const CASTLING_PATH_WHITE_Q_SIDE: u64 = (1 as u64) << (Square::B1 as u32) |
    (1 as u64) << (Square::C1 as u32) |
    (1 as u64) << (Square::D1 as u32);

pub const CASTLING_PATH_BLACK_K_SIDE: u64 = (1 as u64) << (Square::F8 as u32) |
    (1 as u64) << (Square::G8 as u32);
pub const CASTLING_PATH_BLACK_Q_SIDE: u64 = (1 as u64) << (Square::B8 as u32) |
    (1 as u64) << (Square::C8 as u32) |
    (1 as u64) << (Square::D8 as u32);

pub const CASTLING_PATH_WHITE: [u64; CASTLING_SIDES] =
    [CASTLING_PATH_WHITE_K_SIDE, CASTLING_PATH_WHITE_Q_SIDE];
pub const CASTLING_PATH_BLACK: [u64; CASTLING_SIDES] =
    [CASTLING_PATH_BLACK_K_SIDE, CASTLING_PATH_BLACK_Q_SIDE];

pub const CASTLING_PATH: [[u64; CASTLING_SIDES]; PLAYER_CNT] =
    [
        [CASTLING_PATH_WHITE_K_SIDE, CASTLING_PATH_WHITE_Q_SIDE],
        [CASTLING_PATH_BLACK_K_SIDE, CASTLING_PATH_BLACK_Q_SIDE],
    ];


pub const START_BIT_BOARDS: [[BitBoard; PIECE_CNT]; PLAYER_CNT] = [
    [
        START_W_PAWN,
        START_W_KNIGHT,
        START_W_BISHOP,
        START_W_ROOK,
        START_W_QUEEN,
        START_W_KING,
    ],
    [
        START_B_PAWN,
        START_B_KNIGHT,
        START_B_BISHOP,
        START_B_ROOK,
        START_B_QUEEN,
        START_B_KING,
    ],
];

pub const BLANK_BIT_BOARDS: [[BitBoard; PIECE_CNT]; PLAYER_CNT] =
    [[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]];

pub const START_OCC_BOARDS: [BitBoard; PLAYER_CNT] = [START_WHITE_OCC, START_BLACK_OCC];

pub const SQ_DISPLAY_ORDER: [SQ; SQ_CNT] = [56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
    8,  9,  10, 11, 12, 13, 14, 15,
    0,  1,   2,  3,  4,  5,  6,  7];

pub const PIECE_DISPLAYS: [[char; PIECE_CNT]; PLAYER_CNT] = [
    ['P', 'N', 'B', 'R', 'Q', 'K'],
    ['p', 'n', 'b', 'r', 'q', 'k'],
];

pub const FILE_DISPLAYS: [char; FILE_CNT] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANK_DISPLAYS: [char; FILE_CNT] = ['1', '2', '3', '4', '5', '6', '7', '8'];


// Standard FEN strings for benchmarking and testing performance
pub const TEST_FENS: [&str; 5] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "rnr5/pN1kppb1/5n1p/2p5/8/1P2PP2/3B2PP/5KNR w - - 6 27",
    "2r5/1r3k2/p7/2B2N1p/4PP2/1P6/2n3PP/7K b - - 0 40",
    "8/8/5r2/1p5p/6kP/4n3/7K/ w - - 0 57",
    "rnb1k1nr/ppp4p/1b1pppp1/8/4P2P/1Q3N2/PPPP1PP1/RNB1KB1R b KQkq - 1 10",
];


#[inline]
pub fn copy_piece_bbs(
    bbs: &[[BitBoard; PIECE_CNT]; PLAYER_CNT],
) -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
    let new_bbs: [[BitBoard; PIECE_CNT]; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
    new_bbs
}

#[inline]
pub fn return_start_bb() -> [[BitBoard; PIECE_CNT]; PLAYER_CNT] {
    [
        [
            START_W_PAWN,
            START_W_KNIGHT,
            START_W_BISHOP,
            START_W_ROOK,
            START_W_QUEEN,
            START_W_KING,
        ],
        [
            START_B_PAWN,
            START_B_KNIGHT,
            START_B_BISHOP,
            START_B_ROOK,
            START_B_QUEEN,
            START_B_KING,
        ],
    ]
}

#[inline]
pub fn copy_occ_bbs(bbs: &[BitBoard; PLAYER_CNT]) -> [BitBoard; PLAYER_CNT] {
    let new_bbs: [BitBoard; PLAYER_CNT] = unsafe { mem::transmute_copy(bbs) };
    new_bbs
}



/// Returns the other player.
#[inline]
pub fn other_player(p: Player) -> Player {
    // TODO: turn into impl Player
    match p {
        Player::White => Player::Black,
        Player::Black => Player::White,
    }
}

/// Returns the relative value of a piece.
///
/// Used for sorting moves.
#[inline]
pub fn value_of_piece(p: Piece) -> i8 {
    match p {
        Piece::P => 1,
        Piece::N => 3,
        Piece::B => 3,
        Piece::R => 5,
        Piece::Q => 8,
        Piece::K => 0,
    }
}

/// Returns the relative square from a given square.
#[inline]
pub fn relative_square(p: Player, sq: SQ) -> SQ {
    assert!(sq_is_okay(sq));
    sq ^ (p as u8 * 56)
}

#[inline]
pub fn relative_rank_of_sq(p: Player, sq: SQ) -> Rank {
    relative_rank(p, rank_of_sq(sq))
}

#[inline]
pub fn relative_rank(p: Player, rank: Rank) -> Rank {
    ALL_RANKS[((rank as u8) ^ (p as u8 * 7)) as usize]
}


#[inline]
pub fn make_sq(file: File, rank: Rank) -> SQ {
    ((rank as u8).wrapping_shl(3) + (file as u8)) as u8
}

#[inline]
pub fn pawn_push(player: Player) -> i8 {
    match player {
        Player::White => NORTH,
        Player::Black => SOUTH,
    }
}


// For whatever rank the bit is in, gets the whole bitboard
#[inline]
pub fn rank_bb(s: SQ) -> BitBoard {
    RANK_BB[rank_of_sq(s) as usize]
}

#[inline]
pub fn rank_of_sq(s: SQ) -> Rank {
    ALL_RANKS[(s >> 3) as usize]
}

#[inline]
pub fn rank_idx_of_sq(s: SQ) -> u8 {
    (s >> 3) as u8
}

#[inline]
pub fn file_bb(s: SQ) -> u64 {
    FILE_BB[file_of_sq(s) as usize]
}

#[inline]
pub fn file_of_sq(s: SQ) -> File {
    ALL_FILES[(s & 0b0000_0111) as usize]
}

#[inline]
pub fn file_idx_of_sq(s: SQ) -> u8 {
    (s & 0b0000_0111) as u8
}

// Assumes only one bit!
#[inline]
pub fn bb_to_sq(b: BitBoard) -> SQ {
    debug_assert_eq!(bit_twiddles::popcount64(b), 1);
    bit_twiddles::bit_scan_forward(b)
}

// Given a Square (u8) that is valid, returns the bitboard representaton
#[inline]
pub fn sq_to_bb(s: SQ) -> BitBoard {
    assert!(sq_is_okay(s));
    (1 as u64).wrapping_shl(s as u32)
}

// Returns the String of a given square
#[inline]
pub fn parse_sq(s: SQ) -> String {
    assert!(sq_is_okay(s));
    let mut str = String::default();
    str.push(FILE_DISPLAYS[file_of_sq(s) as usize]);
    str.push(RANK_DISPLAYS[rank_of_sq(s) as usize]);
    str
}

// Function to make sure a Square is okay
#[inline]
pub fn sq_is_okay(s: SQ) -> bool {
    s < 64
}


pub fn reverse_bytes(b: BitBoard) -> u64 {
    let mut m: u64 = 0;
    m |= (reverse_byte(((b >> 56) & 0xFF) as u8) as u64) << 56;
    m |= (reverse_byte(((b >> 48) & 0xFF) as u8) as u64) << 48;
    m |= (reverse_byte(((b >> 40) & 0xFF) as u8) as u64) << 40;
    m |= (reverse_byte(((b >> 32) & 0xFF) as u8) as u64) << 32;
    m |= (reverse_byte(((b >> 24) & 0xFF) as u8) as u64) << 24;
    m |= (reverse_byte(((b >> 16) & 0xFF) as u8) as u64) << 16;
    m |= (reverse_byte(((b >> 8) & 0xFF) as u8) as u64) << 8;
    m |= reverse_byte((b & 0xFF) as u8) as u64;
    m
}

pub fn reverse_byte(b: u8) -> u8 {
    let m: u8 = ((0b0000_0001 & b) << 7) | ((0b0000_0010 & b) << 5) | ((0b0000_0100 & b) << 3) |
        ((0b0000_1000 & b) << 1) |
        ((0b0001_0000 & b) >> 1) | ((0b0010_0000 & b) >> 3) |
        ((0b0100_0000 & b) >> 5) | ((0b1000_0000 & b) >> 7);
    m
}

pub fn print_bitboard(input: BitBoard) {
    print_u64(reverse_bytes(input));
}

pub fn print_u64(input: u64) {
    let s = format_u64(input);
    for x in 0..8 {
        let slice = &s[x * 8..(x * 8) + 8];
        println!("{}", slice);
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
