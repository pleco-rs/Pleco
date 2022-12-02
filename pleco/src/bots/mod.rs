//! Contains all of the currently completed standard bots/searchers/AIs.
//!
//! These are mostly for example purposes, to see how one can create a chess AI.

extern crate rand;

pub mod alphabeta;
pub mod iterative_parallel_mvv_lva;
pub mod jamboree;
pub mod minimax;
pub mod parallel_minimax;

use board::Board;
use core::piece_move::*;
use core::score::*;
use tools::eval::*;
use tools::Searcher;

const MAX_PLY: u16 = 4;
const MATE_V: i16 = MATE as i16;
const DRAW_V: i16 = DRAW as i16;
const NEG_INF_V: i16 = NEG_INFINITE as i16;
const INF_V: i16 = INFINITE as i16;

struct BoardWrapper<'a> {
    b: &'a mut Board,
}

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

    fn best_move(board: Board, _depth: u16) -> BitMove {
        let moves = board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }
}

impl Searcher for AlphaBetaSearcher {
    fn name() -> &'static str {
        "AlphaBeta Searcher"
    }

    fn best_move(board: Board, depth: u16) -> BitMove {
        let alpha = NEG_INF_V;
        let beta = INF_V;
        alphabeta::alpha_beta_search(&mut board.shallow_clone(), alpha, beta, depth).bit_move
    }
}

impl Searcher for IterativeSearcher {
    fn name() -> &'static str {
        "Advanced Searcher"
    }

    fn best_move(board: Board, depth: u16) -> BitMove {
        iterative_parallel_mvv_lva::iterative_deepening(&mut board.shallow_clone(), depth)
    }
}

impl Searcher for JamboreeSearcher {
    fn name() -> &'static str {
        "Jamboree Searcher"
    }

    fn best_move(board: Board, depth: u16) -> BitMove {
        let alpha = NEG_INF_V;
        let beta = INF_V;
        jamboree::jamboree(&mut board.shallow_clone(), alpha, beta, depth, 2).bit_move
    }
}

impl Searcher for MiniMaxSearcher {
    fn name() -> &'static str {
        "Simple Searcher"
    }

    fn best_move(board: Board, depth: u16) -> BitMove {
        minimax::minimax(&mut board.shallow_clone(), depth).bit_move
    }
}

impl Searcher for ParallelMiniMaxSearcher {
    fn name() -> &'static str {
        "Parallel Searcher"
    }

    fn best_move(board: Board, depth: u16) -> BitMove {
        parallel_minimax::parallel_minimax(&mut board.shallow_clone(), depth).bit_move
    }
}

#[doc(hidden)]
pub fn eval_board(board: &Board) -> ScoringMove {
    ScoringMove::blank(Eval::eval_low(board) as i16)
}

#[cfg(test)]
mod tests {
    use super::*;

    // We test these, as both algorithms should give the same result no matter if paralleized
    // or not.

    #[test]
    fn minimax_equality() {
        let b = Board::start_pos();
        let b2 = b.shallow_clone();
        assert_eq!(
            MiniMaxSearcher::best_move(b, 5),
            ParallelMiniMaxSearcher::best_move(b2, 5)
        );
    }

    #[test]
    fn alpha_equality() {
        let b = Board::start_pos();
        let b2 = b.shallow_clone();
        assert_eq!(
            AlphaBetaSearcher::best_move(b, 5),
            JamboreeSearcher::best_move(b2, 5)
        );
    }
}
