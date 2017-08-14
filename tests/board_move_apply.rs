extern crate pleco;
extern crate rand;

use pleco::board::Board;
use pleco::templates::{Piece, Player};
use pleco::piece_move::*;
//use piece_move::BitMove;
use std::*;

#[test]
fn random_moves() {
    let mut chess_board = Board::default();
    let mut moves = chess_board.generate_moves();
    let mut i = 0;
    while i < 50 && !moves.is_empty() {
        chess_board.apply_move(moves[rand::random::<usize>() % moves.len()]);
        moves = chess_board.generate_moves();
        i += 1;
    }
}