//! The parallel minimax algorithm.
use board::*;
use core::piece_move::*;
use tools::eval::*;
use super::{BestMove,eval_board};
use core::score::Value;

use rayon;

use std::cmp::max;
#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;


const MAX_PLY: u16 = 5;
const DIVIDE_CUTOFF: usize = 8;

// depth: depth from given
// half_moves: total moves

pub fn parallel_minimax(board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            BestMove::new_none(Value(MATE + (board.depth() as i16)))
        } else {
            BestMove::new_none(Value::DRAW)
        }
    } else {
        parallel_task(&moves, board, max_depth)
    }
}

fn parallel_task(slice: &[BitMove], board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth - 2 || slice.len() <= DIVIDE_CUTOFF {
        let mut best_move = BestMove::new_none(Value::NEG_INFINITE);

        for mov in slice {
            board.apply_move(*mov);
            let returned_move: BestMove = parallel_minimax(board, max_depth)
                .negate()
                .swap_move(*mov);
            board.undo_move();
            best_move = max(returned_move, best_move);
        }
        best_move
    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join(
            || parallel_task(left, &mut left_clone, max_depth),
            || parallel_task(right, board, max_depth),
        );

        max(left_move,right_move)
    }
}