extern crate rusty_chess;

use rusty_chess::board as board;
use self::board::{Board as Board};
use rusty_chess::templates::{Piece, Player};
use rusty_chess::piece_move::*;
use std::*;


#[test]
fn test__init_counts() {
    let board = Board::default();
    assert_eq!(board.count_piece(Player::White, Piece::P), 8);
    assert_eq!(board.count_piece(Player::White, Piece::N), 2);
    assert_eq!(board.count_piece(Player::White, Piece::B), 2);
    assert_eq!(board.count_piece(Player::White, Piece::R), 2);
    assert_eq!(board.count_piece(Player::White, Piece::K), 1);
    assert_eq!(board.count_piece(Player::White, Piece::Q), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::P), 8);
    assert_eq!(board.count_piece(Player::Black, Piece::N), 2);
    assert_eq!(board.count_piece(Player::Black, Piece::B), 2);
    assert_eq!(board.count_piece(Player::Black, Piece::R), 2);
    assert_eq!(board.count_piece(Player::Black, Piece::K), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::Q), 1);
    assert_eq!(board.diagonal_piece_bb(Player::White),0b101100);
    assert_eq!(board.sliding_piece_bb(Player::White),0b10001001);
    assert_eq!(board.count_pieces_player(Player::White),board.count_pieces_player(Player::Black));
    assert_eq!(board.get_occupied(),0xFFFF00000000FFFF);
}

