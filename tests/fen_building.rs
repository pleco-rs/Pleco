extern crate rusty_chess;

use rusty_chess::board as board;
use self::board::{Board as Board, AllBitBoards};
use rusty_chess::templates::{Piece, Player};
use std::*;


// https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
// "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",

// Test base case of working initial board;
#[test]
fn fen_builder_start() {
    // test if no error is thrown in the first place
    let i_board = Board::new_from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    match i_board {
        Ok(x) => {x}
        Err(e) => {
            panic!(e);
        }
    };
    // Test if positions int he right place
    let board = Board::new_from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).unwrap();
    assert_eq!(board.get_occupied(), 0b1111111111111111000000000000000000000000000000001111111111111111);
    assert_eq!(board.get_bitboard(Player::White, Piece::P),  0b0000000000000000000000000000000000000000000000001111111100000000);
    assert_eq!(board.get_bitboard(Player::White, Piece::N),  0b0000000000000000000000000000000000000000000000000000000001000010);
    assert_eq!(board.get_bitboard(Player::White, Piece::B),  0b0000000000000000000000000000000000000000000000000000000000100100);
    assert_eq!(board.get_bitboard(Player::White, Piece::R),  0b0000000000000000000000000000000000000000000000000000000010000001);
    assert_eq!(board.get_bitboard(Player::White, Piece::Q),  0b0000000000000000000000000000000000000000000000000000000000001000);
    assert_eq!(board.get_bitboard(Player::White, Piece::K),  0b0000000000000000000000000000000000000000000000000000000000010000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::P),  0b0000000011111111000000000000000000000000000000000000000000000000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::N),  0b0100001000000000000000000000000000000000000000000000000000000000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::B),  0b0010010000000000000000000000000000000000000000000000000000000000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::R),  0b1000000100000000000000000000000000000000000000000000000000000000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::Q),  0b0000100000000000000000000000000000000000000000000000000000000000);
    assert_eq!(board.get_bitboard(Player::Black, Piece::K),  0b0001000000000000000000000000000000000000000000000000000000000000);
    assert_eq!(board.ply, 0);
}

#[test]
fn multiple_fens() {
    // Test if positions int he right place
    let board = Board::new_from_fen(String::from("r6r/1p2b3/8/8/8/8/P4PPP/1B5R w KQkq - 0 3")).unwrap();
    assert_eq!(board.count_piece(Player::White, Piece::P),  4);
    assert_eq!(board.count_piece(Player::White, Piece::N),  0);
    assert_eq!(board.count_piece(Player::White, Piece::B),  1);
    assert_eq!(board.count_piece(Player::White, Piece::R),  1);
    assert_eq!(board.count_piece(Player::White, Piece::Q),  0);
    assert_eq!(board.count_piece(Player::White, Piece::K),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::P),  1);
    assert_eq!(board.count_piece(Player::Black, Piece::N),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::B),  1);
    assert_eq!(board.count_piece(Player::Black, Piece::R),  2);
    assert_eq!(board.count_piece(Player::Black, Piece::Q),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::K),  0);
    assert_eq!(board.ply, 4);

    let board = Board::new_from_fen(String::from("8/2Q1pk2/nbpppppp/8/8/2K4N/PPPPPPPP/BBB2BBB w ---- a3 0 10")).unwrap();
    assert_eq!(board.count_piece(Player::White, Piece::P),  8);
    assert_eq!(board.count_piece(Player::White, Piece::N),  1);
    assert_eq!(board.count_piece(Player::White, Piece::B),  6);
    assert_eq!(board.count_piece(Player::White, Piece::R),  0);
    assert_eq!(board.count_piece(Player::White, Piece::Q),  1);
    assert_eq!(board.count_piece(Player::White, Piece::K),  1);
    assert_eq!(board.count_piece(Player::Black, Piece::P),  7);
    assert_eq!(board.count_piece(Player::Black, Piece::N),  1);
    assert_eq!(board.count_piece(Player::Black, Piece::B),  1);
    assert_eq!(board.count_piece(Player::Black, Piece::R),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::Q),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::K),  1);
    assert_eq!(board.ply, 18);

    let board = Board::new_from_fen(String::from("8/8/8/8/8/8/8/8 b -kQ- a3 20 1")).unwrap();
    assert_eq!(board.count_piece(Player::White, Piece::P),  0);
    assert_eq!(board.count_piece(Player::White, Piece::N),  0);
    assert_eq!(board.count_piece(Player::White, Piece::B),  0);
    assert_eq!(board.count_piece(Player::White, Piece::R),  0);
    assert_eq!(board.count_piece(Player::White, Piece::Q),  0);
    assert_eq!(board.count_piece(Player::White, Piece::K),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::P),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::N),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::B),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::R),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::Q),  0);
    assert_eq!(board.count_piece(Player::Black, Piece::K),  0);
    assert_eq!(board.ply, 1);
}

