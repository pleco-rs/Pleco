use board::Board;
use timer::Timer;
use piece_move::BitMove;
extern crate rand;
use engine::Searcher;


pub struct RandomBot {
    board: Board,
    timer: Timer,
}

impl Searcher for RandomBot {
    fn name() -> &'static str {
        "Random Searcher"
    }

    fn best_move(board: Board, _timer: &Timer) -> BitMove {
        let moves = board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }

    fn best_move_depth(board: Board, _timer: &Timer, _depth: u16) -> BitMove {
        let moves = board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }
}
