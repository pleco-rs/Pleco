use board::*;
use tools::timer::Timer;
use core::piece_move::*;
use engine::{Searcher,UCILimit};
use board::eval::*;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

use super::super::BestMove;


const MAX_PLY: u16 = 5;


pub struct AlphaBetaBot {
    board: Board,
    timer: Timer,
}

impl Searcher for AlphaBetaBot {
    fn name() -> &'static str {
        "AlphaBeta Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        let alpha = NEG_INFINITY;
        let beta = INFINITY;
        alpha_beta_search(&mut board.shallow_clone(), alpha, beta, max_depth)
            .best_move
            .unwrap()
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

fn eval_board(board: &mut Board) -> BestMove {
    BestMove::new(Eval::eval_low(board))
}
