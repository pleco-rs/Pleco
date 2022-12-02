//! The jamboree algorithm.
use super::alphabeta::alpha_beta_search;
use super::ScoringMove;
use super::*;
use board::*;
use rayon;

const DIVIDE_CUTOFF: usize = 5;
const DIVISOR_SEQ: usize = 4;

pub fn jamboree(
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    depth: u16,
    plys_seq: u16,
) -> ScoringMove {
    assert!(alpha <= beta);
    if depth <= 2 {
        return alpha_beta_search(board, alpha, beta, depth);
    }

    let mut moves = board.generate_scoring_moves();

    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V);
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVISOR_SEQ).min(2) as usize;
    let (seq, non_seq) = moves.split_at_mut(amount_seq);

    let mut best_move: ScoringMove = ScoringMove::blank(alpha);

    for mov in seq {
        board.apply_move(mov.bit_move);
        mov.score = -jamboree(board, -beta, -alpha, depth - 1, plys_seq).score;
        board.undo_move();

        if mov.score > alpha {
            alpha = mov.score;
            if alpha >= beta {
                return *mov;
            }
            best_move = *mov;
        }
    }

    parallel_task(non_seq, board, alpha, beta, depth, plys_seq).max(best_move)
}

fn parallel_task(
    slice: &mut [ScoringMove],
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    depth: u16,
    plys_seq: u16,
) -> ScoringMove {
    if slice.len() <= DIVIDE_CUTOFF {
        let mut best_move: ScoringMove = ScoringMove::blank(alpha);
        for mov in slice {
            board.apply_move(mov.bit_move);
            mov.score = -jamboree(board, -beta, -alpha, depth - 1, plys_seq).score;
            board.undo_move();
            if mov.score > alpha {
                alpha = mov.score;
                if alpha >= beta {
                    return *mov;
                }
                best_move = *mov;
            }
        }
        best_move
    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at_mut(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move): (ScoringMove, ScoringMove) = rayon::join(
            || parallel_task(left, &mut left_clone, alpha, beta, depth, plys_seq),
            || parallel_task(right, board, alpha, beta, depth, plys_seq),
        );

        left_move.max(right_move)
    }
}
