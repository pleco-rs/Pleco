//! The parallel minimax algorithm.
use board::*;
use core::piece_move::*;
use super::*;
use bots::minimax::minimax;

use rayon::prelude::*;

use mucow::MuCow;

pub fn parallel_minimax(board: &mut Board, depth: u16) -> ScoringMove {
    if depth <= 2 {
        return minimax(board, depth);
    }

    let mut moves = board.generate_scoring_moves();
    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V);
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }
    let board_wr: MuCow<Board> = MuCow::Borrowed(board);
    moves.as_mut_slice()
        .par_iter_mut()
//        .with_max_len(6)
        .map_with(board_wr, |b: &mut MuCow<Board>, m: &mut ScoringMove | {
            b.apply_move(m.bit_move);
            m.score = parallel_minimax(&mut *b, depth - 1).negate().score;
            b.undo_move();
            m
        }).max()
        .unwrap()
        .clone()
}
