extern crate pleco;

use std::u16::MAX;

use pleco::board::fen::ALL_FENS;
use pleco::{BitMove, Board};

#[test]
fn pseudolegal_all_fens() {
    for fen in ALL_FENS.iter() {
        let board = Board::from_fen(fen).unwrap();
        pseudolegal_correctness(&board);
    }
}

#[test]
fn pseudolegal_rand() {
    for _x in 0..9 {
        let board = Board::random().one();
        pseudolegal_correctness(&board);
    }
}

#[test]
fn pseudolegal_incheck() {
    let board =
        Board::from_fen("r1b1kb1r/pp2nppp/2pp4/4p3/7P/2Pn2P1/PPq1NPB1/RNB1K1R1 w Qkq - 4 17")
            .unwrap();
    pseudolegal_correctness(&board);
    let board = Board::from_fen("k1r/pp3ppp/n7/3R4/1P5q/1P6/3Kb3/3r4 w - - 1 30").unwrap();
    pseudolegal_correctness(&board);
}

fn pseudolegal_correctness(board: &Board) {
    let pseudo_moves = board.generate_pseudolegal_moves();
    for x in 0..MAX {
        let bit_move = BitMove::new(x);
        if board.pseudo_legal_move(bit_move) {
            if !pseudo_moves.contains(&bit_move) {
                panic!(
                    "\nNot a Pseudo-legal move!\
                    \n  fen: {}\
                    \n  move: {} bits: {:b}\n",
                    board.fen(),
                    bit_move,
                    bit_move.get_raw()
                );
            }
        } else if pseudo_moves.contains(&bit_move) && board.legal_move(bit_move) {
            panic!(
                "\nBoard::pseudolegal move returned false, when it should be true!\
                \n  fen: {}\
                \n  move: {} bits: {:b}\n",
                board.fen(),
                bit_move,
                bit_move.get_raw()
            );
        }
    }
}

#[test]
fn legal_all_fens() {
    for fen in ALL_FENS.iter() {
        let board = Board::from_fen(fen).unwrap();
        legal_correctness(&board);
    }
}

#[test]
fn legal_rand() {
    for _x in 0..10 {
        let board = Board::random().one();
        legal_correctness(&board);
    }
}

fn legal_correctness(board: &Board) {
    let moves = board.generate_moves();
    for m in moves.iter() {
        if !board.pseudo_legal_move(*m) {
            panic!(
                "\nLegal move was not pseudo legal!\
                    \n  fen: {}\
                    \n  move: {} bits: {:b}\n",
                board.fen(),
                m,
                m.get_raw()
            );
        }
    }
}
