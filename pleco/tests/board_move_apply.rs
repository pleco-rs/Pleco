extern crate pleco;
extern crate rand;

use pleco::board::Board;
use pleco::core::piece_move::BitMove;
use std::*;

#[test]
fn random_moves() {
    let mut chess_board = Board::start_pos();
    let mut moves = chess_board.generate_moves();
    let mut i = 0;
    while i < 50 && !moves.is_empty() {
        chess_board.apply_move(moves[rand::random::<usize>() % moves.len()]);
        moves = chess_board.generate_moves();
        i += 1;
    }
}

#[test]
fn apply_null_moves() {
    let null_move = BitMove::null();
    let mut trials = 0;

    while trials < 5 {
        let mut chess_board = Board::default();
        let mut moves = chess_board.generate_moves();
        let mut i = 0;
        while i < 70 && !moves.is_empty() {
            chess_board.apply_move(moves[rand::random::<usize>() % moves.len()]);
            moves = chess_board.generate_moves();
            assert!(!chess_board.legal_move(null_move));
            unsafe {
                if !chess_board.in_check() {
                    chess_board.apply_null_move();
                    chess_board.undo_null_move();
                }
            }

            i += 1;
        }
        trials += 1;
    }
}
