extern crate pleco;

use pleco::board::Board;
use pleco::core::templates::{Piece, Player};


// https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
// "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",

#[test]
fn multiple_fens() {
    // Test if positions int he right place
    let board = Board::new_from_fen("k6r/1p2b3/8/8/8/8/P4KPP/1B5R w KQkq - 0 3");
    assert_eq!(board.count_piece(Player::White, Piece::P), 3);
    assert_eq!(board.count_piece(Player::White, Piece::N), 0);
    assert_eq!(board.count_piece(Player::White, Piece::B), 1);
    assert_eq!(board.count_piece(Player::White, Piece::R), 1);
    assert_eq!(board.count_piece(Player::White, Piece::Q), 0);
    assert_eq!(board.count_piece(Player::White, Piece::K), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::P), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::N), 0);
    assert_eq!(board.count_piece(Player::Black, Piece::B), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::R), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::Q), 0);
    assert_eq!(board.count_piece(Player::Black, Piece::K), 1);


    let board = Board::new_from_fen("8/2Q1pk2/nbpppppp/8/8/2K4N/PPPPPPPP/BBB2BBB w ---- a3 0 10");
    assert_eq!(board.count_piece(Player::White, Piece::P), 8);
    assert_eq!(board.count_piece(Player::White, Piece::N), 1);
    assert_eq!(board.count_piece(Player::White, Piece::B), 6);
    assert_eq!(board.count_piece(Player::White, Piece::R), 0);
    assert_eq!(board.count_piece(Player::White, Piece::Q), 1);
    assert_eq!(board.count_piece(Player::White, Piece::K), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::P), 7);
    assert_eq!(board.count_piece(Player::Black, Piece::N), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::B), 1);
    assert_eq!(board.count_piece(Player::Black, Piece::R), 0);
    assert_eq!(board.count_piece(Player::Black, Piece::Q), 0);
    assert_eq!(board.count_piece(Player::Black, Piece::K), 1);
}

//#[test]
//pub fn test_fens() {
//    for str in &TEST_FENS {
//        Board::new_from_fen(str);
//    }
//}
