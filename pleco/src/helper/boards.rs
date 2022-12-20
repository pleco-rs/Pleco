use core::masks::*;
use core::{file_idx_of_sq, file_of_sq, rank_idx_of_sq, u8_to_u64};
use {File, Player, Rank, SQ};

use super::magic::{bishop_attacks, rook_attacks};

/// Fast lookup Knight moves for each square.
static mut KNIGHT_TABLE: [u64; 64] = [0; 64];
/// Fast lookup King moves for each square.
static mut KING_TABLE: [u64; 64] = [0; 64];
/// Fast lookup distance between each square.
static mut DISTANCE_TABLE: [[u8; 64]; 64] = [[0; 64]; 64];
/// Ring around a certain square
static mut DISTANCE_RING_TABLE: [[u64; 64]; 8] = [[0; 64]; 8];
/// Fast lookup line bitboards for any two squares.
static mut LINE_BITBOARD: [[u64; 64]; 64] = [[0; 64]; 64];
/// Fast lookup bitboards for the squares between any two squares.
static mut BETWEEN_SQUARES_BB: [[u64; 64]; 64] = [[0; 64]; 64];
static mut ADJACENT_FILES_BB: [u64; 8] = [0; 8];
static mut PAWN_ATTACKS_FROM: [[u64; 64]; 2] = [[0; 64]; 2];

static mut PAWN_ATTACKS_SPAN: [[u64; 64]; 2] = [[0; 64]; 2];
static mut FORWARD_FILE_BB: [[u64; 64]; 2] = [[0; 64]; 2];
static mut PASSED_PAWN_MASK: [[u64; 64]; 2] = [[0; 64]; 2];

static mut FORWARD_RANKS_BB: [[u64; PLAYER_CNT]; RANK_CNT] = [[0; PLAYER_CNT]; RANK_CNT];

/// Initialize the static boards.
#[cold]
pub fn init_boards() {
    unsafe {
        gen_king_moves();
        gen_knight_moves();
        gen_distance_table();
        gen_between_and_line_bbs();
        gen_pawn_attacks();
        gen_ring_distance_bb();
        gen_forward_ranks_bb();
        gen_pawn_attacks_span();
    }
}

#[inline(always)]
pub fn knight_moves(sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe { *KNIGHT_TABLE.get_unchecked(sq.0 as usize) }
}

/// Generate King moves `BitBoard` from a source square.
#[inline(always)]
pub fn king_moves(sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe { *KING_TABLE.get_unchecked(sq.0 as usize) }
}

/// Get the distance of two squares.
#[inline(always)]
pub fn distance_of_sqs(sq_one: SQ, sq_two: SQ) -> u8 {
    debug_assert!(sq_one.is_okay());
    debug_assert!(sq_two.is_okay());
    unsafe { DISTANCE_TABLE[sq_one.0 as usize][sq_two.0 as usize] }
}

/// Get the line (diagonal / file / rank) `BitBoard` that two squares both exist on, if it exists.
#[inline(always)]
pub fn line_bb(sq_one: SQ, sq_two: SQ) -> u64 {
    debug_assert!(sq_one.is_okay());
    debug_assert!(sq_two.is_okay());
    unsafe { *(LINE_BITBOARD.get_unchecked(sq_one.0 as usize)).get_unchecked(sq_two.0 as usize) }
}

/// Get the line (diagonal / file / rank) `BitBoard` between two squares, not including the squares, if it exists.
#[inline(always)]
pub fn between_bb(sq_one: SQ, sq_two: SQ) -> u64 {
    debug_assert!(sq_one.is_okay());
    debug_assert!(sq_two.is_okay());
    unsafe {
        *(BETWEEN_SQUARES_BB.get_unchecked(sq_one.0 as usize)).get_unchecked(sq_two.0 as usize)
    }
}

/// Gets the adjacent files `BitBoard` of the square
#[inline(always)]
pub fn adjacent_sq_file(sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe { *ADJACENT_FILES_BB.get_unchecked(sq.file() as usize) }
}

/// Gets the adjacent files `BitBoard` of the file
#[inline(always)]
pub fn adjacent_file(f: File) -> u64 {
    unsafe { *ADJACENT_FILES_BB.get_unchecked(f as usize) }
}
/// Pawn attacks `BitBoard` from a given square, per player.
/// Basically, given square x, returns the BitBoard of squares a pawn on x attacks.
#[inline(always)]
pub fn pawn_attacks_from(sq: SQ, player: Player) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        *PAWN_ATTACKS_FROM
            .get_unchecked(player as usize)
            .get_unchecked(sq.0 as usize)
    }
}

/// Returns if three Squares are in the same diagonal, file, or rank.
#[inline(always)]
pub fn aligned(s1: SQ, s2: SQ, s3: SQ) -> bool {
    (line_bb(s1, s2) & u8_to_u64(s3.0)) != 0
}

/// Returns the ring of bits surrounding the square sq at a specified distance.
///
/// # Safety
///
/// distance must be less than 8, or else a panic will occur.
#[inline(always)]
pub fn ring_distance(sq: SQ, distance: u8) -> u64 {
    debug_assert!(distance <= 7);
    unsafe {
        *DISTANCE_RING_TABLE
            .get_unchecked(distance as usize)
            .get_unchecked(sq.0 as usize)
    }
}

/// Returns the BitBoard of all squares in the rank in front of the given one.
#[inline(always)]
pub fn forward_rank_bb(player: Player, rank: Rank) -> u64 {
    unsafe {
        *FORWARD_RANKS_BB
            .get_unchecked(rank as usize)
            .get_unchecked(player as usize)
    }
}

/// Returns the `BitBoard` of all squares that can be attacked by a pawn
/// of the same color when it moves along its file, starting from the
/// given square. Basically, if the pawn progresses along the same file
/// for the entire game, this bitboard would contain all possible forward squares
/// it could attack
///
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
#[inline(always)]
pub fn pawn_attacks_span(player: Player, sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        *PAWN_ATTACKS_SPAN
            .get_unchecked(player as usize)
            .get_unchecked(sq.0 as usize)
    }
}

/// Returns the BitBoard of all squares in the file in front of the given one.
///
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
#[inline(always)]
pub fn forward_file_bb(player: Player, sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        *FORWARD_FILE_BB
            .get_unchecked(player as usize)
            .get_unchecked(sq.0 as usize)
    }
}

/// Returns a `BitBoard` allowing for testing of the a pawn being a
/// "passed pawn".
/// # Safety
///
/// The Square must be within normal bounds, or else a panic or undefined behaviour may occur.
#[inline(always)]
pub fn passed_pawn_mask(player: Player, sq: SQ) -> u64 {
    debug_assert!(sq.is_okay());
    unsafe {
        *PASSED_PAWN_MASK
            .get_unchecked(player as usize)
            .get_unchecked(sq.0 as usize)
    }
}

// ------------- GENERATION FUNCTIONS -------------

#[cold]
fn gen_knight_moves() {
    unsafe {
        for (index, spot) in KNIGHT_TABLE.iter_mut().enumerate() {
            let mut mask: u64 = 0;
            let file = index % 8;

            // 1 UP   + 2 LEFT
            if file > 1 && index < 56 {
                mask |= 1 << (index + 6);
            }
            // 2 UP   + 1 LEFT
            if file != 0 && index < 48 {
                mask |= 1 << (index + 15);
            }
            // 2 UP   + 1 RIGHT
            if file != 7 && index < 48 {
                mask |= 1 << (index + 17);
            }
            // 1 UP   + 2 RIGHT
            if file < 6 && index < 56 {
                mask |= 1 << (index + 10);
            }
            // 1 DOWN   + 2 RIGHT
            if file < 6 && index > 7 {
                mask |= 1 << (index - 6);
            }
            // 2 DOWN   + 1 RIGHT
            if file != 7 && index > 15 {
                mask |= 1 << (index - 15);
            }
            // 2 DOWN   + 1 LEFT
            if file != 0 && index > 15 {
                mask |= 1 << (index - 17);
            }
            // 1 DOWN   + 2 LEFT
            if file > 1 && index > 7 {
                mask |= 1 << (index - 10);
            }
            *spot = mask;
        }
    }
}

#[cold]
unsafe fn gen_king_moves() {
    for index in 0..64 {
        let mut mask: u64 = 0;
        let file = index % 8;
        // LEFT
        if file != 0 {
            mask |= 1 << (index - 1);
        }
        // RIGHT
        if file != 7 {
            mask |= 1 << (index + 1);
        }
        // UP
        if index < 56 {
            mask |= 1 << (index + 8);
        }
        // DOWN
        if index > 7 {
            mask |= 1 << (index - 8);
        }
        // LEFT UP
        if file != 0 && index < 56 {
            mask |= 1 << (index + 7);
        }
        // LEFT DOWN
        if file != 0 && index > 7 {
            mask |= 1 << (index - 9);
        }
        // RIGHT DOWN
        if file != 7 && index > 7 {
            mask |= 1 << (index - 7);
        }
        // RIGHT UP
        if file != 7 && index < 56 {
            mask |= 1 << (index + 9);
        }
        KING_TABLE[index] = mask;
    }
}

#[cold]
unsafe fn gen_distance_table() {
    for i in 0..64_u8 {
        for j in 0..64_u8 {
            DISTANCE_TABLE[i as usize][j as usize] = (SQ(i)).distance(SQ(j));
        }
    }
}

#[cold]
unsafe fn gen_between_and_line_bbs() {
    for i in 0..64_u8 {
        for j in 0..64_u8 {
            let i_bb: u64 = 1_u64 << i;
            let j_bb: u64 = 1_u64 << j;
            if rook_attacks(0, i) & j_bb != 0 {
                LINE_BITBOARD[i as usize][j as usize] |=
                    (rook_attacks(0, j) & rook_attacks(0, i)) | i_bb | j_bb;
                BETWEEN_SQUARES_BB[i as usize][j as usize] =
                    rook_attacks(i_bb, j) & rook_attacks(j_bb, i);
            } else if bishop_attacks(0, i) & j_bb != 0 {
                LINE_BITBOARD[i as usize][j as usize] |=
                    (bishop_attacks(0, j) & bishop_attacks(0, i)) | i_bb | j_bb;
                BETWEEN_SQUARES_BB[i as usize][j as usize] =
                    bishop_attacks(i_bb, j) & bishop_attacks(j_bb, i);
            } else {
                LINE_BITBOARD[i as usize][j as usize] = 0;
                BETWEEN_SQUARES_BB[i as usize][j as usize] = 0;
            }
        }
    }
}

#[cold]
unsafe fn gen_pawn_attacks() {
    // gen white pawn attacks
    for i in 0..56_u8 {
        let mut bb: u64 = 0;
        if file_of_sq(i) != File::A {
            bb |= u8_to_u64(i + 7)
        }
        if file_of_sq(i) != File::H {
            bb |= u8_to_u64(i + 9)
        }
        PAWN_ATTACKS_FROM[0][i as usize] = bb;
    }

    // Black pawn attacks
    for i in 8..64_u8 {
        let mut bb: u64 = 0;
        if file_of_sq(i) != File::A {
            bb |= u8_to_u64(i - 9)
        }
        if file_of_sq(i) != File::H {
            bb |= u8_to_u64(i - 7)
        }
        PAWN_ATTACKS_FROM[1][i as usize] = bb;
    }
}

#[cold]
unsafe fn gen_ring_distance_bb() {
    for i in 0..64 {
        for j in 0..64 {
            if i != j {
                let dist = DISTANCE_TABLE[i][j] as usize;
                DISTANCE_RING_TABLE[dist - 1][i] |= 1_u64 << (j as usize);
            }
        }
    }
}

#[cold]
unsafe fn gen_forward_ranks_bb() {
    for i in 0..7 {
        FORWARD_RANKS_BB[i + 1][Player::Black as usize] =
            FORWARD_RANKS_BB[i][Player::Black as usize] | RANK_BB[i as usize];
        FORWARD_RANKS_BB[i][Player::White as usize] =
            !FORWARD_RANKS_BB[i + 1][Player::Black as usize];
    }
}

#[cold]
unsafe fn gen_pawn_attacks_span() {
    for p in 0..2 {
        for s in 0..64 {
            FORWARD_FILE_BB[p][s] = FORWARD_RANKS_BB[rank_idx_of_sq(s as u8) as usize][p]
                & FILE_BB[file_idx_of_sq(s as u8) as usize];
            PAWN_ATTACKS_SPAN[p][s] = FORWARD_RANKS_BB[rank_idx_of_sq(s as u8) as usize][p]
                & ADJACENT_FILES_BB[file_idx_of_sq(s as u8) as usize];
            PASSED_PAWN_MASK[p][s] = FORWARD_FILE_BB[p][s] | PAWN_ATTACKS_SPAN[p][s];
        }
    }
}
