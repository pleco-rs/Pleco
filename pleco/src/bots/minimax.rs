use board::*;
use core::piece_move::*;
use board::eval::*;
use super::{BestMove, eval_board};

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

// depth: depth from given
// half_moves: total moves

pub fn minimax(board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {

        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }
    let mut best_value: i16 = NEG_INFINITY;
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
        let returned_move: BestMove = minimax(board, max_depth).negate();
        board.undo_move();
        if returned_move.score > best_value {
            best_value = returned_move.score;
            best_move = Some(mov);
        }
    }
    BestMove {
        best_move: best_move,
        score: best_value,
    }
}