//! Module for evaluating the strength of a current position.

use board::Board;
use std::i16;
use core::templates::*;
use core::bit_twiddles::*;
use core::templates::{PlayerTrait};
use core::masks::*;
use core::sq::SQ;
use core::bitboard::BitBoard;

lazy_static! {
    pub static ref BISHOP_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(BISHOP_POS_ARRAY)), flatten(BISHOP_POS_ARRAY) ];
    pub static ref KNIGHT_POS: [[i16; SQ_CNT]; PLAYER_CNT] = [ flatten(flip(KNIGHT_POS_ARRAY)), flatten(KNIGHT_POS_ARRAY) ];
    pub static ref PAWN_POS:   [[i16; SQ_CNT]; PLAYER_CNT] = [   flatten(flip(PAWN_POS_ARRAY)), flatten(PAWN_POS_ARRAY)   ];
}



const BISHOP_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_8
    [-5, 10, 5, 8, 8, 5, 10, -5],
    [-5, 5, 3, 8, 8, 3, 5, -5],
    [-5, 3, 10, 3, 3, 10, 3, -5],
    [-5, 3, 10, 3, 3, 10, 3, -5],
    [-5, 5, 3, 8, 8, 3, 5, -5],
    [-5, 10, 5, 8, 8, 5, 10, -5],
    [-5, -5, -5, -5, -5, -5, -5, -5], // RANK_1
];

const KNIGHT_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_8
    [-8, 0, 0, 3, 3, 0, 0, -8],
    [-8, 0, 10, 8, 8, 10, 0, -8],
    [-8, 0, 8, 10, 10, 8, 0, -8],
    [-8, 0, 8, 10, 10, 8, 0, -8],
    [-8, 0, 10, 8, 8, 10, 0, -8],
    [-8, 0, 0, 3, 3, 0, 0, -8],
    [-10, -5, -5, -5, -5, -5, -5, -10], // RANK_1
];

const PAWN_POS_ARRAY: [[i16; FILE_CNT]; RANK_CNT] = [
    [0, 0, 0, 0, 0, 0, 0, 0], // RANK_8
    [5, 10, 15, 20, 20, 15, 10, 5],
    [4, 8, 12, 16, 16, 12, 8, 4],
    [0, 6, 9, 10, 10, 9, 6, 0],
    [0, 4, 6, 10, 10, 6, 4, 0],
    [0, 2, 3, 4, 4, 3, 2, 0],
    [0, 0, 0, -5, -5, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0], // RANK_1
];

//  Flips the board, so rank_1 becomes rank_8, rank_8 becomes rank_1, rank_2 becomes rank_7, etc
fn flip(arr: [[i16; FILE_CNT]; RANK_CNT]) -> [[i16; FILE_CNT]; RANK_CNT] {
    let mut new_arr: [[i16; FILE_CNT]; RANK_CNT] = [[0; FILE_CNT]; RANK_CNT];
    for i in 0..RANK_CNT {
        new_arr[i] = arr[7 - i];
    }
    new_arr
}

// Flattens 2D array to a singular 1D array
fn flatten(arr: [[i16; FILE_CNT]; RANK_CNT]) -> [i16; SQ_CNT] {
    let mut new_arr: [i16; SQ_CNT] = [0; SQ_CNT];
    for i in 0..SQ_CNT {
        new_arr[i] = arr[i / 8][i % 8];
    }
    new_arr
}

pub struct Eval {}

trait EvalRuns {
    fn eval_castling<PlayerTrait>(&self) -> i16;
    fn eval_king_pos<PlayerTrait>(&self) -> i16;
    fn eval_bishop_pos<PlayerTrait>(&self) -> i16;
    fn eval_threats<PlayerTrait>(&self) -> i16;
    fn eval_piece_counts<PlayerTrait,PieceTrait>(&self) -> i16;
}


pub const INFINITY: i16 = 30_002;
pub const NEG_INFINITY: i16 = -30_001;
pub const STALEMATE: i16 = 0;

pub const PAWN_VALUE: i16 = 100;
pub const KNIGHT_VALUE: i16 = 300;
pub const BISHOP_VALUE: i16 = 300;
pub const ROOK_VALUE: i16 = 500;
pub const QUEEN_VALUE: i16 = 800;
pub const KING_VALUE: i16 = 350;

pub const CASTLE_ABILITY: i16 = 7;
pub const CASTLE_BONUS: i16 = 20;

pub const KING_BOTTOM: i16 = 11;

pub const MATE: i16 = -25_000;
pub const CHECK: i16 = 20;

// Pawn, Knight, Bishop, Rook, Queen, King
pub const PIECE_VALS: [i16; PIECE_CNT] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];



impl Eval {
    pub fn eval_low(board: &Board) -> i16 {
//        match board.turn() {
//            Player::White => eval_all::<WhiteType>(&board) - eval_all::<BlackType>(&board),
//            Player::Black => eval_all::<BlackType>(&board) - eval_all::<WhiteType>(&board)
//        }
        3
    }
}




fn eval_all<P: PlayerTrait>(board: &Board) -> i16 {
    eval_piece_counts::<P>(board) +
    eval_castling::<P>(board) +
    eval_king_pos::<P>(board) +
    eval_bishop_pos::<P>(board) +
    eval_knight_pos::<P>(board) +
    eval_king_blockers_pinners::<P>(board) +
    eval_pawns::<P>(board)
}


fn eval_piece_counts<P: PlayerTrait>(board: &Board) -> i16 {
    board.count_piece(P::player(), Piece::P) as i16 * PAWN_VALUE +
    board.count_piece(P::player(), Piece::N) as i16 * KNIGHT_VALUE +
    board.count_piece(P::player(), Piece::B) as i16 * BISHOP_VALUE +
    board.count_piece(P::player(), Piece::R) as i16 * ROOK_VALUE +
    board.count_piece(P::player(), Piece::Q) as i16 * QUEEN_VALUE
}

fn eval_castling<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;
    if board.has_castled(P::player()) {
        score += CASTLE_BONUS
    } else {
        if board.can_castle(P::player(), CastleType::KingSide) {
            score += CASTLE_ABILITY
        }
        if board.can_castle(P::player(), CastleType::QueenSide) {
            score += CASTLE_ABILITY
        }
    }
    score
}

fn eval_king_pos<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;
    let us_ksq = board.king_sq(P::player());
    if us_ksq.rank_of_sq() == Rank::R1 || us_ksq.rank_of_sq() == Rank::R8 {
        score += KING_BOTTOM
    }

    if board.in_check() && P::player() == board.turn() {
        score -= CHECK
    }

    let bb_around_us: BitBoard = board.magic_helper.king_moves(us_ksq) & board.get_occupied_player(P::player());
    score += bb_around_us.count_bits() as i16 * 9;

    score
}

fn eval_bishop_pos<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;
    let mut us_b = board.piece_bb(P::player(), Piece::B);
    while us_b.is_not_empty() {
        let lsb = us_b.lsb();
        score += BISHOP_POS[P::player() as usize][lsb.bb_to_sq().0 as usize];
        us_b &= !lsb;
    }

    if board.count_piece(P::player(), Piece::B) > 1 {
        score += 19
    }

    score
}


fn eval_knight_pos<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;
    let mut us_b = board.piece_bb(P::player(), Piece::N);
    while us_b.is_not_empty() {
        let lsb = us_b.lsb();
        score += KNIGHT_POS[P::player() as usize][lsb.bb_to_sq().0 as usize];
        us_b &= !lsb;
    }

    score
}

fn eval_king_blockers_pinners<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;

    let blockers: BitBoard = board.all_pinned_pieces(P::player());

    let them_blockers: BitBoard = blockers & board.get_occupied_player(P::opp_player());

    // Our pieces blocking a check on their king
    let us_blockers: BitBoard = blockers & board.get_occupied_player(P::player());

    score += 25 * us_blockers.count_bits() as i16;

    score += 6 * them_blockers.count_bits() as i16;

    score
}

fn eval_pawns<P: PlayerTrait>(board: &Board) -> i16 {
    let mut score: i16 = 0;

    let pawns_bb: BitBoard = board.piece_bb(P::player(), Piece::P);
    let mut bb = pawns_bb;
    let mut file_counts: [u8; FILE_CNT] = [0; FILE_CNT];

    let mut sqs_defended: BitBoard = BitBoard(0);

    while bb.is_not_empty() {
        let lsb = bb.lsb();
        let sq = lsb.bb_to_sq();
        sqs_defended |= board.magic_helper.pawn_attacks_from(sq, P::player());
        file_counts[(sq.0 % 8) as usize] += 1;
        score += PAWN_POS[P::player() as usize][sq.0 as usize];
        bb &= !lsb;
    }

    // Add score for squares attacked by pawns
    score += sqs_defended.count_bits() as i16;

    // Add score for pawns defending other pawns
    sqs_defended &= pawns_bb;
    score += 3 * sqs_defended.count_bits() as i16;

    for i in 0..FILE_CNT {
        if file_counts[i] > 1 {
            score -= (file_counts[i] * 3) as i16;
        }
        if i > 0 && i < 7 && file_counts[i] > 0 {
            if file_counts[i - 1] != 0 {
                if file_counts[i + 1] != 0 {
                    score += 7;
                } else {
                    score += 3;
                }
            } else if file_counts[i + 1] != 0 {
                score += 3;
            } else {
                score -= 4;
            }
        }
    }

    score

}

// Without MonoMorphization 100 times
//  9,145 ns
//  9,192 ns
//  9,047 ns

// With MonoMorphizing 100 times
//  8,645 ns
//  8,438 ns


// TODO Mobility Bonus
