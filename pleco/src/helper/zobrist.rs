use {Player,SQ,PieceType,BitBoard};
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

/// Zobrist key for having no pawns;
static mut ZOBRIST_NO_PAWNS: u64 = 0; // 8

/// initialize the zobrist hash
#[cold]
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

        for cr in 0..ALL_CASTLING_RIGHTS {
            ZOBRIST_CASTLE[cr] = 0;

            // We do this as having all castling rights is similar to having all individual
            // castling rights. So, ALL_CASTLE = CASLTE_Q_W ^ CASLTE_Q_B ^ CASLTE_K_W ^ CASLTE_K_B
            let mut b = BitBoard(cr as u64);
            while let Some(s) = b.pop_some_lsb() {
                let mut k: u64 = ZOBRIST_CASTLE[1 << s.0 as usize];
                if k == 0 {k = rng.rand();}
                ZOBRIST_CASTLE[cr] ^= k;
            }
        }
        ZOBRIST_SIDE = rng.rand();
        ZOBRIST_NO_PAWNS = rng.rand();
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

#[inline(always)]
pub fn z_no_pawns() -> u64 {
    unsafe { ZOBRIST_NO_PAWNS }
}
