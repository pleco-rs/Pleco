//! The minimax algorithm.
use board::*;
use super::*;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

use std::cmp::max;

// depth: depth from given
// half_moves: total moves

pub fn minimax(board: &mut Board, max_depth: u16) -> ScoringMove {
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V + (board.depth() as i16));
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }
    let mut best_move = ScoringMove::blank(NEG_INF_V);
    for mov in moves {
        board.apply_move(mov);
        let returned_move: ScoringMove = minimax(board, max_depth)
            .negate()
            .swap_move(mov);

        board.undo_move();
        best_move = max(returned_move, best_move);
    }
    best_move
}

pub fn minimax2(board: &mut Board, max_depth: u16) -> ScoringMove {
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    let moves = board.generate_scoring_moves();
    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V + (board.depth() as i16));
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }

    moves.into_iter()
        .map(|mut m: ScoringMove| {
            board.apply_move(m.bit_move);
            m.score = -minimax2(board, max_depth - 1).score;
            board.undo_move();
            m
        }).max()
        .unwrap()

//    let mut best_move = ScoringMove::blank(NEG_INF_V);
//    for mov in moves {
//        board.apply_move(mov);
//        let returned_move: ScoringMove = minimax(board, max_depth)
//            .negate()
//            .swap_move(mov);
//
//        board.undo_move();
//        best_move = max(returned_move, best_move);
//    }
//    best_move
}