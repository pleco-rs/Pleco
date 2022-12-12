extern crate pleco;
extern crate rand;

use pleco::board::Board;
use std::*;

#[test]
fn threefold_repetition() {
    let mut chess_board = Board::start_pos();
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("e2e4");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("e7e5");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("f1c4");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("f8c5");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("c4f1");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("c5f8");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("f1c4");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("f8c5");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("c4f1");
    assert!(!chess_board.threefold_repetition());
    assert!(!chess_board.stalemate());
    chess_board.apply_uci_move("c5f8");
    assert!(chess_board.threefold_repetition());
    assert!(chess_board.stalemate());
}
