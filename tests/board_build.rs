extern crate rusty_chess;

use rusty_chess::board as board;
use self::board::{Board as Board, AllBitBoards};
use rusty_chess::templates::{Piece, Player};
use rusty_chess::movegen::*;
use rusty_chess::piece_move::*;
use std::*;
use rusty_chess::bit_twiddles::*;


#[test]
fn fen_builder() {
    // test if Works in the first place
    let i_board = Board::new_from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
    match i_board {
        Ok(x) => {}
        Err(e) => {
            panic!(e);
        }
    };
    let board = Board::new_from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).unwrap();
    assert_eq!(board.get_occupied(), 0b1111111111111111000000000000000000000000000000001111111111111111);
//    assert_eq!(board.get_bitboard(Player::White, Piece::P).unwrap(),  0b0000000000000000000000000000000000000000000000001111111100000000);
//    assert_eq!(board.get_bitboard(Player::White, Piece::N).unwrap(),  0b0000000000000000000000000000000000000000000000000000000001000010);
//    assert_eq!(board.get_bitboard(Player::White, Piece::B).unwrap(),  0b0000000000000000000000000000000000000000000000000000000000100100);
//    assert_eq!(board.get_bitboard(Player::White, Piece::R).unwrap(),  0b0000000000000000000000000000000000000000000000000000000010000001);
//    assert_eq!(board.get_bitboard(Player::White, Piece::Q).unwrap(),  0b0000000000000000000000000000000000000000000000000000000000001000);
//    assert_eq!(board.get_bitboard(Player::White, Piece::K).unwrap(),  0b0000000000000000000000000000000000000000000000000000000000010000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::P).unwrap(),  0b0000000011111111000000000000000000000000000000000000000000000000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::N).unwrap(),  0b0100001000000000000000000000000000000000000000000000000000000000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::B).unwrap(),  0b0010010000000000000000000000000000000000000000000000000000000000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::R).unwrap(),  0b1000000100000000000000000000000000000000000000000000000000000000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::Q).unwrap(),  0b0000100000000000000000000000000000000000000000000000000000000000);
//    assert_eq!(board.get_bitboard(Player::Black, Piece::K).unwrap(),  0b0001000000000000000000000000000000000000000000000000000000000000);

}

#[test]
fn check_two_piece_one_square() {
    let board = Board::new();
    let xor = board.bit_boards.into_iter().fold(0, |sum, x| sum ^ x);
    let or = board.bit_boards.into_iter().fold(0, |sum, x| sum | x);
    assert_eq!(or, xor);
}


#[test]
fn test_counts() {
    let board = Board::new();

    let count_w_p = board.count_piece(Player::White, Piece::P);
    assert_eq!(count_w_p, 8);

    let count_w_n = board.count_piece(Player::White, Piece::N);
    assert_eq!(count_w_n, 2);

    let count_w_b = board.count_piece(Player::White, Piece::B);
    assert_eq!(count_w_b, 2);

    let count_w_r = board.count_piece(Player::White, Piece::R);
    assert_eq!(count_w_r, 2);

    let count_w_k = board.count_piece(Player::White, Piece::K);
    assert_eq!(count_w_k, 1);

    let count_w_q = board.count_piece(Player::White, Piece::Q);
    assert_eq!(count_w_q, 1);

    let count_b_p = board.count_piece(Player::Black, Piece::P);
    assert_eq!(count_b_p, 8);

    let count_b_n = board.count_piece(Player::Black, Piece::N);
    assert_eq!(count_b_n, 2);

    let count_b_b = board.count_piece(Player::Black, Piece::B);
    assert_eq!(count_b_b, 2);

    let count_b_r = board.count_piece(Player::Black, Piece::R);
    assert_eq!(count_b_r, 2);

    let count_b_k = board.count_piece(Player::Black, Piece::K);
    assert_eq!(count_b_k, 1);

    let count_b_q = board.count_piece(Player::Black, Piece::Q);
    assert_eq!(count_b_q, 1);
}

