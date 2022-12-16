//! The minimax algorithm.
use super::*;
use board::*;

pub fn minimax(board: &mut Board, depth: u16) -> ScoringMove {
    if depth == 0 {
        return eval_board(board);
    }

    board
        .generate_scoring_moves()
        .into_iter()
        .map(|mut m: ScoringMove| {
            board.apply_move(m.bit_move);
            m.score = -minimax(board, depth - 1).score;
            board.undo_move();
            m
        })
        .max()
        .unwrap_or_else(|| match board.in_check() {
            true => ScoringMove::blank(-MATE_V),
            false => ScoringMove::blank(DRAW_V),
        })
}

pub fn minimax_eval_bitmove(board: &mut Board, bm: BitMove, depth: u16) -> i16 {
    board.apply_move(bm);
    let out = -minimax(board, depth).score;
    board.undo_move();
    out
}
