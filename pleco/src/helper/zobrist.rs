use {Player,SQ,PieceType};
use tools::prng::PRNG;
use core::masks::*;

/// Seed for the Zobrist's pseudo-random number generator.
const ZOBRIST_SEED: u64 = 23_081;

/// Zobrist key for each piece on each square.
static mut ZOBRIST_PIECE_SQUARE: [[[u64; PIECE_TYPE_CNT]; PLAYER_CNT]; SQ_CNT] =
    [[[0; PIECE_TYPE_CNT]; PLAYER_CNT]; SQ_CNT];

/// Zobrist key for each possible en-passant capturable file.
static mut ZOBRIST_ENPASSANT: [u64; FILE_CNT] = [0; FILE_CNT]; // 8 * 8

/// Zobrist key for each possible castling rights.
static mut ZOBRIST_CASTLE: [u64; ALL_CASTLING_RIGHTS] =[0; ALL_CASTLING_RIGHTS]; // 8 * 4

/// Zobrist key for the side to move.
static mut ZOBRIST_SIDE: u64 = 0; // 8

pub fn init_zobrist() {
    let mut rng = PRNG::init(ZOBRIST_SEED);

    unsafe {
        for i in 0..SQ_CNT {
            for j in 0..PIECE_TYPE_CNT {
                ZOBRIST_PIECE_SQUARE[i][0][j] = rng.rand();
                ZOBRIST_PIECE_SQUARE[i][1][j] = rng.rand();
            }
        }

        for i in 0..FILE_CNT {
            ZOBRIST_ENPASSANT[i] = rng.rand()
        }

        ZOBRIST_CASTLE[0] = 0;

        for i in 1..ALL_CASTLING_RIGHTS {
            ZOBRIST_CASTLE[i] = rng.rand()
        }

        ZOBRIST_SIDE = rng.rand();
    }
}

#[inline(always)]
pub fn z_square(sq: SQ, player: Player, piece: PieceType) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        ZOBRIST_PIECE_SQUARE[sq.0 as usize][player as usize][piece as usize]
    }
}

#[inline(always)]
pub fn z_ep(sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        *ZOBRIST_ENPASSANT.get_unchecked(sq.file() as usize)
    }
}

#[inline(always)]
pub fn z_castle(castle: u8) -> u64 {
    debug_assert!((castle as usize) < ALL_CASTLING_RIGHTS);
    unsafe {
        *ZOBRIST_CASTLE.get_unchecked(castle as usize)
    }
}

#[inline(always)]
pub fn z_side() -> u64 {
    unsafe { ZOBRIST_SIDE }
}