//! Contains all of the currently completed standard bots/searchers/AIs.
//!
//! These are mostly for example purposes, to see how one can create a chess AI.

extern crate rand;

pub mod minimax;
pub mod parallel_minimax;
pub mod alphabeta;
pub mod jamboree;
pub mod iterative_parallel_mvv_lva;

use core::piece_move::BitMove;
use tools::{Searcher,UCILimit};
use board::Board;
use board::eval::*;

use std::cmp::{Ordering,PartialEq,PartialOrd,Ord};

const MAX_PLY: u16 = 4;

/// Searcher that randomly chooses a move. The fastest, yet dumbest, searcher we have to offer.
pub struct RandomBot {}

/// Searcher that uses a MiniMax algorithm to search for a best move.
pub struct MiniMaxSearcher {}
/// Searcher that uses a MiniMax algorithm to search for a best move, but does so in parallel.
pub struct ParallelMiniMaxSearcher {}
/// Searcher that uses an alpha-beta algorithm to search for a best move.
pub struct AlphaBetaSearcher {}
/// Searcher that uses a modified alpha-beta algorithm to search for a best move, but does so in parallel.
/// The specific name of this algorithm is called "jamboree".
pub struct JamboreeSearcher {}
/// Modified `JamboreeSearcher` that uses the parallel alpha-beta algorithm. Improves upon `JamboreeSearcher` by
/// adding iterative deepening with an aspiration window, MVV-LVA move ordering, as well as a qscience search.
pub struct IterativeSearcher {}

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

/// Used by the Searchers to keep track of a move's strength.
#[derive(Eq, Copy, Clone)]
pub struct BestMove {
    best_move: Option<BitMove>,
    score: i16,
}

impl BestMove {
    #[inline(always)]
    pub fn new_none(score: i16) -> Self {
        BestMove {
            best_move: None,
            score: score,
        }
    }

    #[inline(always)]
    pub fn negate(mut self) -> Self {
        self.score = self.score.wrapping_neg();
        self
    }

    #[inline(always)]
    pub fn swap_move(mut self, bitmove: BitMove) -> Self {
        self.best_move = Some(bitmove);
        self
    }
}

impl Ord for BestMove {
    fn cmp(&self, other: &BestMove) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for BestMove {
    fn partial_cmp(&self, other: &BestMove) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BestMove {
    fn eq(&self, other: &BestMove) -> bool {
        self.score == other.score
    }
}


#[doc(hidden)]
pub fn eval_board(board: &Board) -> BestMove {
    BestMove::new_none(Eval::eval_low(board))
}


