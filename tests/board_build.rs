extern crate pleco;

use pleco::board as board;
use self::board::{Board as Board};
use pleco::core::templates::*;
use pleco::core::piece_move;
use pleco::core::piece_move::*;
use pleco::*;


#[test]
fn test_init_counts() {
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
    assert_eq!(board.diagonal_piece_bb(Player::White).0,0b101100);
    assert_eq!(board.sliding_piece_bb(Player::White).0,0b10001001);
    assert_eq!(board.count_pieces_player(Player::White),board.count_pieces_player(Player::Black));
    assert_eq!(board.get_occupied().0,0xFFFF00000000FFFF);
}


#[test]
fn basic_move_apply() {
    let mut b = Board::default();
    let p1 = PreMoveInfo {
        src: SQ(12),
        dst: SQ(28),
        flags: MoveFlag::DoublePawnPush
    };
    let m = BitMove::init(p1);
    b.apply_move(m);
    let p2 = PreMoveInfo {
        src: SQ(51),
        dst: SQ(35),
        flags: MoveFlag::DoublePawnPush
    };
    let m = BitMove::init(p2);
    b.apply_move(m);
    let p3 = PreMoveInfo {
        src: SQ(28),
        dst: SQ(35),
        flags: MoveFlag::Capture {ep_capture: false}
    };
    let m = BitMove::init(p3);
    b.apply_move(m);
    assert_eq!(b.count_piece(Player::Black,Piece::P),7);
    b.undo_move();
    assert_eq!(b.count_piece(Player::Black,Piece::P),8);
    assert!(!b.in_check());
}



#[test]
fn move_seq_1() {
    let mut b = board::Board::default();
    let p = PreMoveInfo {
        src: SQ(12),
        dst: SQ(28),
        flags: MoveFlag::DoublePawnPush
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(51),
        dst: SQ(35),
        flags: MoveFlag::DoublePawnPush
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(28),
        dst: SQ(35),
        flags: MoveFlag::Capture {ep_capture: false}
    };
    let m = BitMove::init(p);
    b.apply_move(m);

    let p = PreMoveInfo {
        src: SQ(59),
        dst: SQ(35),
        flags: MoveFlag::Capture {ep_capture: false}
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(5),
        dst: SQ(12),
        flags: MoveFlag::QuietMove,
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(35),
        dst: SQ(8),
        flags: MoveFlag::Capture {ep_capture: false}
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(6),
        dst: SQ(21),
        flags: MoveFlag::QuietMove
    };
    let m = BitMove::init(p);
    b.apply_move(m);

    let p = piece_move::PreMoveInfo {
        src: SQ(60),
        dst: SQ(59),
        flags: piece_move::MoveFlag::QuietMove
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(4),
        dst: SQ(7),
        flags: MoveFlag::Castle{king_side: true}
    };
    let m = BitMove::init(p);
    b.apply_move(m);
}