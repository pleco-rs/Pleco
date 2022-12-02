extern crate pleco;

use pleco::board::{Board, RandBoard};
use pleco::bot_prelude::*;
use pleco::tools::Searcher;

#[test]
fn test_all_bot() {
    for _x in 0..3 {
        let board: Board = RandBoard::default().one();
        RandomBot::best_move(board.shallow_clone(), 4);
        MiniMaxSearcher::best_move(board.shallow_clone(), 4);
        AlphaBetaSearcher::best_move(board.shallow_clone(), 4);
        ParallelMiniMaxSearcher::best_move(board.shallow_clone(), 4);
        JamboreeSearcher::best_move(board.shallow_clone(), 4);
    }
}
