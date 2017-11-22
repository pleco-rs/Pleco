//! Contains all of the currently completed and experimental bots.

extern crate rand;

pub mod minimax;
pub mod parallel_minimax;
pub mod alphabeta;
pub mod jamboree;
pub mod iterative_parallel_mvv_lva;

use core::piece_move::BitMove;
use engine::{Searcher,UCILimit};
use board::Board;
use board::eval::*;

const MAX_PLY: u16 = 4;

pub struct AlphaBetaSearcher {}
pub struct IterativeSearcher {}
pub struct JamboreeSearcher {}
pub struct MiniMaxSearcher {}
pub struct ParallelMiniMaxSearcher {}
pub struct RandomBot {}

impl Searcher for RandomBot {
    fn name() -> &'static str {
        "Random Searcher"
    }

    fn best_move(board: Board, _limit: UCILimit) -> BitMove {
        let moves = board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }
}

impl Searcher for AlphaBetaSearcher {
    fn name() -> &'static str {
        "AlphaBeta Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        let alpha = NEG_INFINITY;
        let beta = INFINITY;
        alphabeta::alpha_beta_search(&mut board.shallow_clone(), alpha, beta, max_depth)
            .best_move
            .unwrap()
    }
}

impl Searcher for IterativeSearcher {
    fn name() -> &'static str {
        "Advanced Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        iterative_parallel_mvv_lva::iterative_deepening(board, max_depth)
    }
}

impl Searcher for JamboreeSearcher {
    fn name() -> &'static str {
        "Jamboree Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        let alpha = NEG_INFINITY;
        let beta = INFINITY;
        jamboree::jamboree(&mut board.shallow_clone(), alpha, beta, max_depth, 2)
            .best_move
            .unwrap()
    }
}

impl Searcher for MiniMaxSearcher {
    fn name() -> &'static str {
        "Simple Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        minimax::minimax( &mut board.shallow_clone(),  max_depth).best_move.unwrap()
    }
}


impl Searcher for ParallelMiniMaxSearcher {
    fn name() -> &'static str {
        "Parallel Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        parallel_minimax::parallel_minimax(&mut board.shallow_clone(),  max_depth)
            .best_move
            .unwrap()
    }
}


pub struct BestMove {
    best_move: Option<BitMove>,
    score: i16,
}

impl BestMove {
    pub fn new(score: i16) -> Self {
        BestMove {
            best_move: None,
            score: score,
        }
    }

    pub fn negate(mut self) -> Self {
        self.score = self.score.wrapping_neg();
        self
    }
}

pub fn eval_board(board: &Board) -> BestMove {
    BestMove::new(Eval::eval_low(board))
}


