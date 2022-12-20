//! Module for simply evaluating the strength of a current position.
//!
//! This is a VERY basic evaluation, and while decent, it certainly isn't anything exceptional.

use core::bitboard::BitBoard;
use core::masks::*;
use core::mono_traits::*;
use core::score::Value;
use core::*;
use std::i32;
use Board;

lazy_static! {
    pub static ref PAWN_POS: [[i32; SQ_CNT]; PLAYER_CNT] =
        [flatten(flip(PAWN_POS_ARRAY)), flatten(PAWN_POS_ARRAY)];
}

const PAWN_POS_ARRAY: [[i32; FILE_CNT]; RANK_CNT] = [
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
fn flip(arr: [[i32; FILE_CNT]; RANK_CNT]) -> [[i32; FILE_CNT]; RANK_CNT] {
    let mut new_arr: [[i32; FILE_CNT]; RANK_CNT] = [[0; FILE_CNT]; RANK_CNT];
    for i in 0..RANK_CNT {
        new_arr[i] = arr[7 - i];
    }
    new_arr
}

// Flattens 2D array to a singular 1D array
fn flatten(arr: [[i32; FILE_CNT]; RANK_CNT]) -> [i32; SQ_CNT] {
    let mut new_arr: [i32; SQ_CNT] = [0; SQ_CNT];
    for i in 0..SQ_CNT {
        new_arr[i] = arr[i / 8][i % 8];
    }
    new_arr
}

/// A simple evaluation structure. This is included as an example, and shouldn't
/// necessarily be used inside serious chess engines.
///
/// ```
/// use pleco::tools::eval::Eval;
/// use pleco::Board;
///
/// let board = Board::start_pos();
/// let score = Eval::eval_low(&board);
/// println!("Score: {}", score);
/// ```
pub struct Eval;

trait EvalRuns {
    fn eval_castling<PlayerTrait>(&self) -> i32;
    fn eval_king_pos<PlayerTrait>(&self) -> i32;
    fn eval_bishop_pos<PlayerTrait>(&self) -> i32;
    fn eval_threats<PlayerTrait>(&self) -> i32;
    fn eval_piece_counts<PlayerTrait, PieceTrait>(&self) -> i32;
}

const INFINITY: i32 = 30_001;
const NEG_INFINITY: i32 = -30_001;
const STALEMATE: i32 = 0;
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 800;
const KING_VALUE: i32 = 350;
const CASTLE_ABILITY: i32 = 7;
const CASTLE_BONUS: i32 = 20;
const KING_BOTTOM: i32 = 8;
const MATE: i32 = -25_000;
const CHECK: i32 = 14;

// Pawn, Knight, Bishop, Rook, Queen, King
pub const PIECE_VALS: [i32; PIECE_TYPE_CNT] = [
    0,
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
    0,
];

impl Eval {
    /// Evaluates the score of a `Board` for the current side to move.
    pub fn eval_low(board: &Board) -> Value {
        match board.turn() {
            Player::White => {
                eval_all::<WhiteType>(board) - eval_all::<BlackType>(board)
                    + board.non_pawn_material(Player::White)
                    - board.non_pawn_material(Player::Black)
            }
            Player::Black => {
                eval_all::<BlackType>(board) - eval_all::<WhiteType>(board)
                    + board.non_pawn_material(Player::Black)
                    - board.non_pawn_material(Player::White)
            }
        }
    }
}

fn eval_all<P: PlayerTrait>(board: &Board) -> Value {
    if board.rule_50() >= 50 {
        return MATE;
    }
    eval_piece_counts::<P>(board)
        + eval_castling::<P>(board)
        + eval_king_pos::<P>(board)
        + eval_bishop_pos::<P>(board)
        + eval_king_blockers_pinners::<P>(board)
        + eval_pawns::<P>(board)
}

fn eval_piece_counts<P: PlayerTrait>(board: &Board) -> i32 {
    board.count_piece(P::player(), PieceType::P) as i32 * PAWN_VALUE
}

fn eval_castling<P: PlayerTrait>(board: &Board) -> i32 {
    let mut score: i32 = 0;

    if board.can_castle(P::player(), CastleType::KingSide) {
        score += CASTLE_ABILITY
    }
    if board.can_castle(P::player(), CastleType::QueenSide) {
        score += CASTLE_ABILITY
    }
    score
}

fn eval_king_pos<P: PlayerTrait>(board: &Board) -> i32 {
    let mut score: i32 = 0;
    let us_ksq = board.king_sq(P::player());

    if board.in_check() && P::player() == board.turn() {
        score -= CHECK
    }

    let bb_around_us: BitBoard =
        board.magic_helper.king_moves(us_ksq) & board.get_occupied_player(P::player());
    score += bb_around_us.count_bits() as i32 * 9;

    score
}

fn eval_bishop_pos<P: PlayerTrait>(board: &Board) -> i32 {
    let mut score: i32 = 0;

    if board.count_piece(P::player(), PieceType::B) > 1 {
        score += 19
    }

    score
}

fn eval_king_blockers_pinners<P: PlayerTrait>(board: &Board) -> i32 {
    let mut score: i32 = 0;

    let blockers: BitBoard = board.all_pinned_pieces(P::player());

    let them_blockers: BitBoard = blockers & board.get_occupied_player(P::opp_player());

    // Our pieces blocking a check on their king
    let us_blockers: BitBoard = blockers & board.get_occupied_player(P::player());

    score += 18 * us_blockers.count_bits() as i32;

    score += 6 * them_blockers.count_bits() as i32;

    score
}

fn eval_pawns<P: PlayerTrait>(board: &Board) -> i32 {
    let mut score: i32 = 0;

    let pawns_bb: BitBoard = board.piece_bb(P::player(), PieceType::P);
    let mut bb = pawns_bb;
    let mut file_counts: [u8; FILE_CNT] = [0; FILE_CNT];

    let mut sqs_defended: BitBoard = BitBoard(0);

    while bb.is_not_empty() {
        let lsb = bb.lsb();
        let sq = lsb.to_sq();
        sqs_defended |= board.magic_helper.pawn_attacks_from(sq, P::player());
        file_counts[(sq.0 % 8) as usize] += 1;
        score += PAWN_POS[P::player() as usize][sq.0 as usize];
        bb &= !lsb;
    }

    // Add score for squares attacked by pawns
    score += sqs_defended.count_bits() as i32;

    // Add score for pawns defending other pawns
    sqs_defended &= pawns_bb;
    score += 3 * sqs_defended.count_bits() as i32;

    for i in 0..FILE_CNT {
        if file_counts[i] > 1 {
            score -= (file_counts[i] * 3) as i32;
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
