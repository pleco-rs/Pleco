extern crate pleco;

use self::board::Board;
use pleco::board;
use pleco::core::piece_move;
use pleco::core::piece_move::*;
use pleco::core::*;
use pleco::*;

#[test]
fn test_init_counts() {
    let board = Board::start_pos();
    assert_eq!(board.count_piece(Player::White, PieceType::P), 8);
    assert_eq!(board.count_piece(Player::White, PieceType::N), 2);
    assert_eq!(board.count_piece(Player::White, PieceType::B), 2);
    assert_eq!(board.count_piece(Player::White, PieceType::R), 2);
    assert_eq!(board.count_piece(Player::White, PieceType::K), 1);
    assert_eq!(board.count_piece(Player::White, PieceType::Q), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::P), 8);
    assert_eq!(board.count_piece(Player::Black, PieceType::N), 2);
    assert_eq!(board.count_piece(Player::Black, PieceType::B), 2);
    assert_eq!(board.count_piece(Player::Black, PieceType::R), 2);
    assert_eq!(board.count_piece(Player::Black, PieceType::K), 1);
    assert_eq!(board.count_piece(Player::Black, PieceType::Q), 1);
    assert_eq!(board.diagonal_piece_bb(Player::White).0, 0b101100);
    assert_eq!(board.sliding_piece_bb(Player::White).0, 0b10001001);
    assert_eq!(
        board.count_pieces_player(Player::White),
        board.count_pieces_player(Player::Black)
    );
    assert_eq!(board.occupied().0, 0xFFFF00000000FFFF);
    assert_eq!(board.count_all_pieces(), 32);
}

#[test]
fn basic_move_apply() {
    let mut b = Board::start_pos();
    let p1 = PreMoveInfo {
        src: SQ(12),
        dst: SQ(28),
        flags: MoveFlag::DoublePawnPush,
    };
    let m = BitMove::init(p1);
    b.apply_move(m);
    let p2 = PreMoveInfo {
        src: SQ(51),
        dst: SQ(35),
        flags: MoveFlag::DoublePawnPush,
    };
    let m = BitMove::init(p2);
    b.apply_move(m);
    let p3 = PreMoveInfo {
        src: SQ(28),
        dst: SQ(35),
        flags: MoveFlag::Capture { ep_capture: false },
    };
    let m = BitMove::init(p3);
    b.apply_move(m);
    assert_eq!(b.count_piece(Player::Black, PieceType::P), 7);
    b.undo_move();
    assert_eq!(b.count_piece(Player::Black, PieceType::P), 8);
    assert!(!b.in_check());
}

#[test]
fn move_seq_1() {
    let mut b = board::Board::start_pos();
    let p = PreMoveInfo {
        src: SQ(12),
        dst: SQ(28),
        flags: MoveFlag::DoublePawnPush,
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(51),
        dst: SQ(35),
        flags: MoveFlag::DoublePawnPush,
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(28),
        dst: SQ(35),
        flags: MoveFlag::Capture { ep_capture: false },
    };
    let m = BitMove::init(p);
    b.apply_move(m);

    let p = PreMoveInfo {
        src: SQ(59),
        dst: SQ(35),
        flags: MoveFlag::Capture { ep_capture: false },
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
        flags: MoveFlag::Capture { ep_capture: false },
    };
    let m = BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(6),
        dst: SQ(21),
        flags: MoveFlag::QuietMove,
    };
    let m = BitMove::init(p);
    b.apply_move(m);

    let p = piece_move::PreMoveInfo {
        src: SQ(60),
        dst: SQ(59),
        flags: piece_move::MoveFlag::QuietMove,
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    let p = PreMoveInfo {
        src: SQ(4),
        dst: SQ(7),
        flags: MoveFlag::Castle { king_side: true },
    };
    let m = BitMove::init(p);
    b.apply_move(m);
}
