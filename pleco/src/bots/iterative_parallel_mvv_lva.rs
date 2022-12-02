//! The iterative jamboree algorithm.
use super::*;
use board::*;
use core::piece_move::BitMove;
use core::*;
use rayon;

const MAX_PLY: u16 = 5;

const DIVIDE_CUTOFF: usize = 6;
const DIVISOR_SEQ: usize = 5;

//                            0   1   2   3   4   5   6   7   8   9
static PLYS_SEQ: [u16; 10] = [0, 1, 2, 2, 2, 2, 2, 3, 3, 3];

pub fn iterative_deepening(board: &mut Board, max_depth: u16) -> BitMove {
    // for each level from 1 to max depth, search the node and return the best move and score
    // Once we have reached ply 2, keep the score (say x), c
    //       continue onto previous ply with alpha = x - 33 and beta = x + 33
    //       If ply n + 1 returns with score y && y > x + 33 || y < x - 33
    //          if fail low, redo with alpha = -inf
    //          if fail high, redo with beta = inf
    //       now if re-search fails, do a full ply search with alpha = -inf and beta = inf
    //

    let mut i = 2;
    let mut alpha: i16 = NEG_INF_V;
    let mut beta: i16 = INF_V;

    // Create a dummy best_move
    let mut best_move = ScoringMove::blank(NEG_INF_V);

    // Loop until max_depth is reached
    while i <= max_depth {
        // clone the board
        let mut b = board.shallow_clone();

        let returned_b_move = jamboree(&mut b, alpha, beta, i, PLYS_SEQ[i as usize]);
        if i >= 2 {
            if returned_b_move.score > beta {
                beta = INF_V;
            } else if returned_b_move.score < alpha {
                alpha = NEG_INF_V;
            } else {
                if returned_b_move.bit_move != BitMove::null() {
                    alpha = returned_b_move.score - 34;
                    beta = returned_b_move.score + 34;
                    best_move = returned_b_move;
                }
                i += 1;
            }
        }
    }

    best_move.bit_move
}

fn jamboree(
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    max_depth: u16,
    plys_seq: u16,
) -> ScoringMove {
    assert!(alpha <= beta);
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    if board.depth() >= max_depth - plys_seq {
        return alpha_beta_search(board, alpha, beta, max_depth);
    }

    let mut moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(MATE_V + (board.depth() as i16));
        } else {
            return ScoringMove::blank(DRAW_V);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVIDE_CUTOFF) as usize;

    let (seq, non_seq) = moves.split_at_mut(amount_seq);

    let mut best_move: BitMove = BitMove::null();
    let mut best_value: i16 = NEG_INF_V;
    for mov in seq {
        board.apply_move(*mov);
        let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
        board.undo_move();

        if return_move.score > best_value {
            best_move = *mov;
            best_value = return_move.score;

            if return_move.score > alpha {
                alpha = return_move.score;
                best_move = *mov;
            }

            if alpha >= beta {
                return ScoringMove {
                    bit_move: *mov,
                    score: alpha,
                };
            }
        }
    }

    let returned_move = parallel_task(non_seq, board, alpha, beta, max_depth, plys_seq);

    if returned_move.score > alpha {
        returned_move
    } else {
        ScoringMove {
            bit_move: best_move,
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
) -> ScoringMove {
    let mut best_move: BitMove = BitMove::null();
    if slice.len() <= DIVIDE_CUTOFF {
        for mov in slice {
            board.apply_move(*mov);
            let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
            board.undo_move();

            if return_move.score > alpha {
                alpha = return_move.score;
                best_move = *mov;
            }

            if alpha >= beta {
                return ScoringMove {
                    bit_move: *mov,
                    score: alpha,
                };
            }
        }
    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join(
            || parallel_task(left, &mut left_clone, alpha, beta, max_depth, plys_seq),
            || parallel_task(right, board, alpha, beta, max_depth, plys_seq),
        );

        if left_move.score > alpha {
            alpha = left_move.score;
            best_move = left_move.bit_move;
        }
        if right_move.score > alpha {
            alpha = right_move.score;
            best_move = right_move.bit_move;
        }
    }
    ScoringMove {
        bit_move: best_move,
        score: alpha,
    }
}

fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> ScoringMove {
    if board.depth() >= max_depth {
        if board.in_check() || board.piece_last_captured().is_some() {
            return quiescence_search(board, alpha, beta, max_depth + 2);
        }
        return eval_board(board);
    }

    // Futility Pruning
    if max_depth > 2
        && board.depth() == max_depth - 1
        && board.piece_last_captured().is_none()
        && !board.in_check()
    {
        let eval = eval_board(board);
        if eval.score + 100 < alpha {
            return quiescence_search(board, alpha, beta, max_depth + 1);
        }
    }

    let moves = board.generate_moves();

    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(MATE_V + (board.depth() as i16));
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

fn quiescence_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> ScoringMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = if board.in_check() {
        board.generate_moves()
    } else {
        board.generate_moves_of_type(GenTypes::Captures)
    };

    if moves.is_empty() {
        if board.in_check() {
            return ScoringMove::blank(-MATE_V + (board.depth() as i16));
        }
        return eval_board(board);
    }
    let mut best_move: BitMove = BitMove::null();
    for mov in moves {
        board.apply_move(mov);

        let return_move = { quiescence_search(board, -beta, -alpha, max_depth) }.negate();

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

fn q_science_criteria(m: BitMove, _board: &Board) -> bool {
    m.is_capture()
}

fn mvv_lva_sort(moves: &mut [BitMove], board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).type_of();

        if a.is_capture() {
            board.captured_piece(*a).value() - piece.value()
        } else if piece == PieceType::P {
            if a.is_double_push().0 {
                -2
            } else {
                -3
            }
        } else {
            -4
        }
    })
}
