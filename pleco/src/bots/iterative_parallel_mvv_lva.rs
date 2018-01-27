//! The iterative jamboree algorithm.
use board::*;
use core::*;
use core::piece_move::BitMove;
use tools::eval::*;
use super::{BestMove,eval_board};
use core::score::Value;
use rayon;


#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

const MAX_PLY: u16 = 5;

const DIVIDE_CUTOFF: usize = 6;
const DIVISOR_SEQ: usize = 5;

//                            0   1   2   3   4   5   6   7   8   9
static PLYS_SEQ: [u16; 10] = [0, 1, 2, 2, 2, 2, 2, 3, 3, 3];

pub fn iterative_deepening(board: Board, max_depth: u16) -> BitMove {
    // for each level from 1 to max depth, search the node and return the best move and score
    // Once we have reached ply 2, keep the score (say x), c
    //       continue onto previous ply with alpha = x - 33 and beta = x + 33
    //       If ply n + 1 returns with score y && y > x + 33 || y < x - 33
    //          if fail low, redo with alpha = -inf
    //          if fail high, redo with beta = inf
    //       now if re-search fails, do a full ply search with alpha = -inf and beta = inf
    //

    let mut i = 2;
    let mut alpha: i16 = NEG_INFINITY;
    let mut beta: i16 = INFINITY;

    // Create a dummy best_move
    let mut best_move = BestMove::new_none(Value::NEG_INFINITE);

    // Loop until max_depth is reached
    while i <= max_depth {
        // clone the board
        let mut b = board.shallow_clone();

        let returned_b_move = jamboree(&mut b, alpha, beta, i, PLYS_SEQ[i as usize]);
        if i >= 2 {
            if returned_b_move.score.0 > beta {
                beta = INFINITY;
            } else if returned_b_move.score.0 < alpha {
                alpha = NEG_INFINITY;
            } else {
                if returned_b_move.best_move.is_some() {
                    alpha = returned_b_move.score.0 - 34;
                    beta = returned_b_move.score.0 + 34;
                    best_move = returned_b_move;
                }
                i += 1;
            }
        }
    }
    if best_move.best_move.is_none() {
        println!("{}, i = {}", best_move.score.0, i);
    }
    best_move.best_move.unwrap()
}



fn jamboree(
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    max_depth: u16,
    plys_seq: u16,
) -> BestMove {
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
            return BestMove::new_none(Value(MATE + (board.depth() as i16)));
        } else {
            return BestMove::new_none(Value::DRAW);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVIDE_CUTOFF) as usize;

    let (seq, non_seq) = moves.split_at_mut(amount_seq);


    let mut best_move: Option<BitMove> = None;
    let mut best_value: i16 = NEG_INFINITY;
    for mov in seq {
        board.apply_move(*mov);
        let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
        board.undo_move();


        if return_move.score.0 > best_value {
                best_move = Some(*mov);
            best_value = return_move.score.0;

            if return_move.score.0 > alpha {
                alpha = return_move.score.0;
                best_move = Some(*mov);
            }

            if alpha >= beta {
                return BestMove {
                    best_move: Some(*mov),
                    score: Value(alpha),
                };
            }
        }
    }

    let returned_move = parallel_task(non_seq, board, alpha, beta, max_depth, plys_seq);

    if returned_move.score.0 > alpha {
        returned_move
    } else {
        BestMove {
            best_move: best_move,
            score: Value(best_value),
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

            if return_move.score.0 > alpha {
                alpha = return_move.score.0;
                best_move = Some(*mov);
            }

            if alpha >= beta {
                return BestMove {
                    best_move: Some(*mov),
                    score: Value(alpha),
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

        if left_move.score.0 > alpha {
            alpha = left_move.score.0;
            best_move = left_move.best_move;
        }
        if right_move.score.0 > alpha {
            alpha = right_move.score.0;
            best_move = right_move.best_move;
        }
    }
    BestMove {
        best_move: best_move,
        score: Value(alpha),
    }

}

fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {

    if board.depth() >= max_depth {
        if board.in_check() || board.piece_last_captured().is_some() {
            return quiescence_search(board, alpha, beta, max_depth + 2);
        }
        return eval_board(board);
    }

    // Futility Pruning
    if max_depth > 2 && board.depth() == max_depth - 1 &&
        board.piece_last_captured().is_none() && !board.in_check()
    {
        let eval = eval_board(board);
        if eval.score.0 + 100 < alpha {
            return quiescence_search(board, alpha, beta, max_depth + 1);
        }
    }

    let moves = board.generate_moves();


    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new_none(Value(MATE + (board.depth() as i16)));
        } else {
            return BestMove::new_none(Value::DRAW);
        }
    }



    let mut best_move: Option<BitMove> = None;

    for mov in moves {
        board.apply_move(mov);
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();

        if return_move.score.0 > alpha {
            alpha = return_move.score.0;
            best_move = Some(mov);
        }

        if alpha >= beta {
            return BestMove {
                best_move: Some(mov),
                score: Value(alpha),
            };
        }
    }

    BestMove {
        best_move: best_move,
        score: Value(alpha),
    }
}

fn quiescence_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {
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
            return BestMove::new_none(Value(MATE + (board.depth() as i16)));
        }
        return eval_board(board);
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);

        let return_move = { quiescence_search(board, -beta, -alpha, max_depth) }.negate();

        board.undo_move();

        if return_move.score.0 > alpha {
            alpha = return_move.score.0;
            best_move = Some(mov);
        }

        if alpha >= beta {
            return BestMove {
                best_move: Some(mov),
                score: Value(alpha),
            };
        }
    }

    BestMove {
        best_move: best_move,
        score: Value(alpha),
    }
}

fn q_science_criteria(m: BitMove, _board: &Board) -> bool {
    m.is_capture()
}

fn mvv_lva_sort(moves: &mut [BitMove], board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture() {
            board.captured_piece(*a).unwrap().value() - piece.value()
        } else if piece == Piece::P {
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
