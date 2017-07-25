use piece_move::BitMove;
use timer::Timer;
use board::Board;

pub trait Searcher {
    fn best_move(board: Board, timer: Timer) -> BitMove;
    fn name() -> &'static str;
}