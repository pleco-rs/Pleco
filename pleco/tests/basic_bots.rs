extern crate pleco;


use pleco::engine::Searcher;
use pleco::bot_prelude::*;
use pleco::board::{Board,RandBoard};



#[test]
fn test_all_bot() {
    for _x in 0..5 {
        let board: Board = RandBoard::default().one();
        RandomBot::best_move_depth(board.shallow_clone(), 4);
        MiniMaxSearcher::best_move_depth(board.shallow_clone(), 4);
        AlphaBetaSearcher::best_move_depth(board.shallow_clone(), 4);
        ParallelMiniMaxSearcher::best_move_depth(board.shallow_clone(), 4);
        JamboreeSearcher::best_move_depth(board.shallow_clone(), 4);
    }
}