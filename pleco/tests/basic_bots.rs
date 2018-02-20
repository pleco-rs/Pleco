extern crate pleco;


use pleco::tools::Searcher;
use pleco::bot_prelude::*;
use pleco::board::{Board,RandBoard};

use pleco::tools::eval::Eval;


#[test]
fn test_all_bot() {
    for _x in 0..3 {
        let board: Board = RandBoard::default().one();
        RandomBot::best_move_depth(board.shallow_clone(), 4);
        MiniMaxSearcher::best_move_depth(board.shallow_clone(), 4);
        AlphaBetaSearcher::best_move_depth(board.shallow_clone(), 4);
        ParallelMiniMaxSearcher::best_move_depth(board.shallow_clone(), 4);
        JamboreeSearcher::best_move_depth(board.shallow_clone(), 4);
    }
}

#[test]
fn test_search() {
    let mut b = Board::default();
    for x in 0..15 {
        println!("Score: {}", Eval::eval_low(&b));
        b.pretty_print();
        let m = JamboreeSearcher::best_move_depth(b.shallow_clone(), 5);
        b.apply_move(m);
    }
}