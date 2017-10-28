use board::Board;
use core::piece_move::BitMove;
extern crate rand;
use engine::{Searcher,UCILimit};
use timer::Timer;


pub struct RandomBot {
    board: Board,
    timer: Timer,
}

impl Searcher for RandomBot {
    fn name() -> &'static str {
        "Random Searcher"
    }

    fn best_move(board: Board, _limit: UCILimit) -> BitMove {
        let moves = board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }

}
