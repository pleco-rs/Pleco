//! The jamboree algorithm.
use board::*;
use core::piece_move::*;
use board::eval::*;
use super::{BestMove,eval_board};

use rayon;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

const DIVIDE_CUTOFF: usize = 5;
const DIVISOR_SEQ: usize = 4;

// depth: depth from given
// half_moves: total moves

pub fn jamboree(board: &mut Board, mut alpha: i16, beta: i16,
                max_depth: u16, plys_seq: u16) -> BestMove
{
    assert!(alpha <= beta);
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    if board.depth() >= max_depth - plys_seq {
        return alpha_beta_search(board, alpha, beta, max_depth);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVIDE_CUTOFF) as usize;
    let (seq, non_seq) = moves.split_at(amount_seq);

    let mut best_move: Option<BitMove> = None;
    let mut best_value: i16 = NEG_INFINITY;
    for mov in seq {
        board.apply_move(*mov);
        let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
        board.undo_move();


        if return_move.score > best_value {
            best_move = Some(*mov);
            best_value = return_move.score;

            if return_move.score > alpha {
                alpha = return_move.score;
                best_move = Some(*mov);
            }

            if alpha >= beta {
                return BestMove {
                    best_move: Some(*mov),
                    score: alpha,
                };
            }
        }
    }

    let returned_move = parallel_task(non_seq, board, alpha, beta, max_depth, plys_seq);

    if returned_move.score > alpha {
        returned_move
    } else {
        BestMove {
            best_move: best_move,
            score: best_value,
        }
    }
}

fn parallel_task(
    slice: &[BitMove],
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    max_depth: u16,
    plys_seq: u16,
) -> BestMove {
    let mut best_move: Option<BitMove> = None;
    if slice.len() <= DIVIDE_CUTOFF {
        for mov in slice {
            board.apply_move(*mov);
            let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
            board.undo_move();

            if return_move.score > alpha {
                alpha = return_move.score;
                best_move = Some(*mov);
            }

            if alpha >= beta {
                return BestMove {
                    best_move: Some(*mov),
                    score: alpha,
                };
            }
        }

    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join (
            || parallel_task(left, &mut left_clone, alpha, beta, max_depth, plys_seq),
            || parallel_task(right, board, alpha, beta, max_depth, plys_seq));

        if left_move.score > alpha {
            alpha = left_move.score;
            best_move = left_move.best_move;
        }
        if right_move.score > alpha {
            alpha = right_move.score;
            best_move = right_move.best_move;
        }
    }
    BestMove {
        best_move: best_move,
        score: alpha,
    }

}



fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();

    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(-STALEMATE);
        }
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();

        if return_move.score > alpha {
            alpha = return_move.score;
            best_move = Some(mov);
        }

        if alpha >= beta {
            return BestMove {
                best_move: Some(mov),
                score: alpha,
            };
        }
    }

    BestMove {
        best_move: best_move,
        score: alpha,
    }
}