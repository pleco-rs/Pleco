use board::*;
use timer::*;
use piece_move::*;
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

    fn best_move(board: Board, timer: Timer) -> BitMove {
        let bot = RandomBot { board: board, timer: timer};
        let moves = bot.board.generate_moves();
        moves[rand::random::<usize>() % moves.len()]
    }
}