extern crate pleco;

use pleco::board::Board;
use pleco::core::{PieceType, Player};

// https://chessprogramming.wikispaces.com/Forsyth-Edwards+Notation
// "r1bqkbr1/1ppppp1N/p1n3pp/8/1P2PP2/3P4/P1P2nPP/RNBQKBR1 b KQkq -",

#[test]
fn basic_fen() {
    // Test if positions int he right place
    let board = Board::from_fen("k6r/1p2b3/8/8/8/8/P4KPP/1B5R w KQkq - 0 3").unwrap();
    assert_eq!(board.count_piece(Player::White, PieceType::P), 3);
    assert_eq!(board.count_piece(Player::White, PieceType::N), 0);
    assert_eq!(board.count_piece(Player::White, PieceType::B), 1);
    assert_eq!(board.count_piece(Player::White, PieceType::R), 1);
    assert_eq!(board.count_piece(Player::White, PieceType::Q), 0);
    assert_eq!(board.count_piece(Player::White, PieceType::K), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::P), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::N), 0);
    assert_eq!(board.count_piece(Player::Black, PieceType::B), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::R), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::Q), 0);
    assert_eq!(board.count_piece(Player::Black, PieceType::K), 1);

    let board = Board::from_fen("8/2Q1pk2/nbpppppp/8/8/2K4N/PPPPPPPP/BBB2BBB w - - 0 10").unwrap();
    assert_eq!(board.count_piece(Player::White, PieceType::P), 8);
    assert_eq!(board.count_piece(Player::White, PieceType::N), 1);
    assert_eq!(board.count_piece(Player::White, PieceType::B), 6);
    assert_eq!(board.count_piece(Player::White, PieceType::R), 0);
    assert_eq!(board.count_piece(Player::White, PieceType::Q), 1);
    assert_eq!(board.count_piece(Player::White, PieceType::K), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::P), 7);
    assert_eq!(board.count_piece(Player::Black, PieceType::N), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::B), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::R), 0);
    assert_eq!(board.count_piece(Player::Black, PieceType::Q), 0);
    assert_eq!(board.count_piece(Player::Black, PieceType::K), 1);
}

#[test]
fn all_fens() {
    for fen in pleco::board::fen::ALL_FENS.iter() {
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(*fen, board.fen());
    }
}

#[test]
fn rank8_zero_fen() {
    let fen = "8/2Q1pk2/nbpppppp/8/8/2K4N/PPPPPPPP/BBB2BBB w - - 0 10";
    let board = Board::from_fen(fen).unwrap();
    assert_eq!(fen, board.fen());
}
