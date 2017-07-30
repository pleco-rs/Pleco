use piece_move::BitMove;
use timer::Timer;
use board::Board;

pub trait Searcher {
    fn best_move(board: Board, timer: &Timer) -> BitMove;
    fn best_move_depth(board: Board, timer: &Timer, max_depth: u16) -> BitMove;
    fn name() -> &'static str;
}