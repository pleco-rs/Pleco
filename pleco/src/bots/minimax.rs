//! The minimax algorithm.
use board::*;
use board::eval::*;
use super::{BestMove, eval_board};

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
            return BestMove::new_none(MATE + (board.depth() as i16));
        } else {
            return BestMove::new_none(STALEMATE);
        }
    }
    let mut best_move = BestMove::new_none(NEG_INFINITY);
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