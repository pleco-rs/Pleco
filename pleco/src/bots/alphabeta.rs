//! The alpha-beta algorithm.
use board::*;
use core::piece_move::*;
use super::*;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

use super::{ScoringMove, eval_board};


const MAX_PLY: u16 = 5;

pub fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> ScoringMove {

    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();

    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V + board.depth() as i16);
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }

    let mut best_move: BitMove = BitMove::null();
    for mov in moves {
        board.apply_move(mov);
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();
        if return_move.score > alpha {
            alpha = return_move.score;
            best_move = mov;
        }
        if alpha >= beta {
            return ScoringMove {
                bit_move: mov,
                score: alpha,
            };
        }
    }

    ScoringMove {
        bit_move: best_move,
        score: alpha,
    }
}