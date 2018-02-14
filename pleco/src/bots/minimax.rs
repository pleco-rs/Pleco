//! The minimax algorithm.
use board::*;
use super::{BestMove, eval_board};
use core::score::*;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

use std::cmp::max;

// depth: depth from given
// half_moves: total moves

pub fn minimax(board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new_none(MATE + (board.depth() as i32));
        } else {
            return BestMove::new_none(DRAW);
        }
    }
    let mut best_move = BestMove::new_none(NEG_INFINITE);
    for mov in moves {
        board.apply_move(mov);
        let returned_move: BestMove = minimax(board, max_depth)
            .negate()
            .swap_move(mov);

        board.undo_move();
        best_move = max(returned_move, best_move);
    }
    best_move
}