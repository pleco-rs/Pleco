extern crate rusty_chess;

use rusty_chess::board::{Board, AllBitBoards};
use rusty_chess::templates::{Piece, Player};
use rusty_chess::movegen as movegen;
use rusty_chess::piece_move::*;
use rusty_chess::bit_twiddles::*;


#[test]
fn test_pawn_gen() {
    let board = Board::new();
    let vector = movegen::get_pseudo_moves(&board, Player::White);
    assert_eq!(vector.len(), 16);
}

