extern crate pleco;
extern crate rand;

use pleco::board::Board;
use pleco::core::piece_move::BitMove;
use pleco::core::sq::SQ;
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

#[test]
fn double_pawn_push_sets_ep_square() {
    let fen1 = "r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2";
    let mut board = Board::from_fen(fen1).unwrap();

    let move_to_play = BitMove::make(BitMove::FLAG_DOUBLE_PAWN, SQ::D2, SQ::D4);
    board.apply_move(move_to_play);

    let expected_fen = "r1bqkbnr/pppppppp/2n5/8/3PP3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2";
    assert_eq!(expected_fen, board.fen());
}
