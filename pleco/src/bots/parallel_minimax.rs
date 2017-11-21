use board::*;
use core::piece_move::*;
use board::eval::*;
use super::{BestMove,eval_board};

use rayon;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;


const MAX_PLY: u16 = 5;
const DIVIDE_CUTOFF: usize = 8;

// depth: depth from given
// half_moves: total moves

pub fn parallel_minimax(board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            BestMove::new(MATE + (board.depth() as i16))
        } else {
            BestMove::new(STALEMATE)
        }
    } else {
        parallel_task(&moves, board, max_depth)
    }
}

fn parallel_task(slice: &[BitMove], board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth - 2 || slice.len() <= DIVIDE_CUTOFF {
        let mut best_value: i16 = NEG_INFINITY;
        let mut best_move: Option<BitMove> = None;
        for mov in slice {
            board.apply_move(*mov);
            let returned_move: BestMove = parallel_minimax(board, max_depth).negate();
            board.undo_move();
            if returned_move.score > best_value {
                best_value = returned_move.score;
                best_move = Some(*mov);
            }
        }
        BestMove {
            best_move: best_move,
            score: best_value,
        }
    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join(
            || parallel_task(left, &mut left_clone, max_depth),
            || parallel_task(right, board, max_depth),
        );

        if left_move.score > right_move.score {
            left_move
        } else {
            right_move
        }
    }
}