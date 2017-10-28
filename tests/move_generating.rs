extern crate pleco;

use pleco::board::*;
use pleco::core::templates::*;
use pleco::core::piece_move::*;
use pleco::tools::gen_rand_legal_board;



#[test]
fn test_movegen_captures() {
    let mut vec = Vec::new();
    for i in 0..6 {
        let mut b = gen_rand_legal_board();
        if !b.in_check() {
            vec.push(b);
        }
    }
    vec.iter().for_each(|b| {
        let moves = b.generate_moves_of_type(GenTypes::Captures);
        for m in moves {
            assert!(m.is_capture());
        }
    })
}

#[test]
fn test_movegen_quiets() {
    let mut vec = Vec::new();
    for i in 0..6 {
        let mut b = gen_rand_legal_board();
        if !b.in_check() {
            vec.push(b);
        }
    }
    vec.iter().for_each(|b| {
        let moves = b.generate_moves_of_type(GenTypes::Quiets);
        for m in moves {
            assert!(!m.is_capture());
            assert!(b.captured_piece(m).is_none());
        }
    })
}


// Testing with no flags and bit input
#[test]
fn bit_move_position() {
    let bits: u16 = 0b0000111011010000;
    let bit_move = BitMove::new(bits);
    assert_eq!(bit_move.get_src(), 0b010000);
    assert_eq!(bit_move.get_dest(), 0b111011);
    assert!(bit_move.is_quiet_move());
    assert!(!bit_move.is_promo());
    assert!(!bit_move.is_capture());
    assert!(!bit_move.is_castle());
    assert!(!bit_move.is_king_castle());
    assert!(!bit_move.is_queen_castle());
    assert!(!bit_move.is_double_push().0);
    assert!(!bit_move.is_en_passant());
}

#[test]
fn test_opening_position() {
    let b = Board::default();
    let moves = b.generate_moves();
    assert_eq!(moves.len(), (8 * 2) + (2 * 2));
}

#[test]
fn test_move_permutations() {
    let moves = all_move_flags();
    for move_flag in moves {
        let pre_move_info = PreMoveInfo {
            src: 9,
            dst: 42,
            flags: move_flag,
        };
        let move_info = BitMove::init(pre_move_info);
        assert_eq!(move_flag == MoveFlag::QuietMove, move_info.is_quiet_move());
        assert_eq!(
            move_flag == MoveFlag::Castle { king_side: true } ||
                move_flag == MoveFlag::Castle { king_side: false },
            move_info.is_castle()
        );
        assert_eq!(
            move_flag == MoveFlag::Castle { king_side: true },
            move_info.is_king_castle()
        );
        assert_eq!(
            move_flag == MoveFlag::Castle { king_side: false },
            move_info.is_queen_castle()
        );
        assert_eq!(
            move_flag == MoveFlag::DoublePawnPush,
            move_info.is_double_push().0
        );
        assert_eq!(
            move_flag == MoveFlag::Capture { ep_capture: true },
            move_info.is_en_passant()
        );
    }
}

// Test all Promotion Moves for correct Piece Placement
#[test]
fn bit_move_promoions() {
    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::P,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::N,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::N);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::B,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::B);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::R,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::R);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::K,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: Piece::Q,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::P,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::N,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::N);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::B,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::B);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::R,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::R);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::K,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: Piece::Q,
    };
    let pre_move_info = PreMoveInfo {
        src: 9,
        dst: 42,
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), Piece::Q);
}

fn all_move_flags() -> Vec<MoveFlag> {
    let mut move_flags = Vec::new();
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::P,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::N,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::B,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::R,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::K,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: Piece::Q,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::P,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::N,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::B,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::R,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::K,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: Piece::Q,
    });
    move_flags.push(MoveFlag::Castle { king_side: true });
    move_flags.push(MoveFlag::Castle { king_side: false });
    move_flags.push(MoveFlag::Capture { ep_capture: true });
    move_flags.push(MoveFlag::Capture { ep_capture: false });
    move_flags.push(MoveFlag::DoublePawnPush);
    move_flags.push(MoveFlag::QuietMove);
    move_flags
}