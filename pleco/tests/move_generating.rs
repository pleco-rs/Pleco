extern crate pleco;

use pleco::board::{Board, RandBoard};
use pleco::core::piece_move::*;
use pleco::core::*;
use pleco::SQ;
use std::str::FromStr;

#[test]
fn test_movegen_captures() {
    let vec = RandBoard::default().no_check().many(9);

    vec.iter().for_each(|b| {
        let moves = b.generate_moves_of_type(GenTypes::Captures);
        for m in moves {
            if !m.is_promo() {
                assert!(m.is_capture());
                assert!(b.captured_piece(m).is_real());
            }
        }
    })
}

#[test]
fn test_movegen_quiets() {
    let vec = RandBoard::default().no_check().many(6);

    vec.iter().for_each(|b| {
        let moves = b.generate_moves_of_type(GenTypes::Quiets);
        for m in moves {
            if !m.is_promo() && !m.is_castle() {
                assert!(!m.is_capture());
                assert!(!b.captured_piece(m).is_real());
            }
        }
    })
}

#[test]
fn test_movegen_quiet_checks() {
    let vec = RandBoard::default().no_check().many(5);

    vec.iter().for_each(|b| {
        b.generate_moves_of_type(GenTypes::QuietChecks);
    })
}

// Testing with no flags and bit input
#[test]
fn bit_move_position() {
    let bits: u16 = 0b0000111011010000;
    let bit_move = BitMove::new(bits);
    assert_eq!(bit_move.get_src().0, 0b010000);
    assert_eq!(bit_move.get_dest().0, 0b111011);
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
    let b = Board::start_pos();
    let moves = b.generate_moves();
    assert_eq!(moves.len(), (8 * 2) + (2 * 2));
}

#[test]
fn test_move_permutations() {
    let moves = all_move_flags();
    for move_flag in moves {
        let pre_move_info = PreMoveInfo {
            src: SQ(9),
            dst: SQ(42),
            flags: move_flag,
        };
        let move_info = BitMove::init(pre_move_info);
        assert_eq!(move_flag == MoveFlag::QuietMove, move_info.is_quiet_move());
        assert_eq!(
            move_flag == MoveFlag::Castle { king_side: true }
                || move_flag == MoveFlag::Castle { king_side: false },
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
        prom: PieceType::P,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: PieceType::N,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::N);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: PieceType::B,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::B);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: PieceType::R,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::R);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: PieceType::K,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);

    let move_flag = MoveFlag::Promotion {
        capture: true,
        prom: PieceType::Q,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::P,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::N,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::N);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::B,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::B);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::R,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::R);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::K,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);

    let move_flag = MoveFlag::Promotion {
        capture: false,
        prom: PieceType::Q,
    };
    let pre_move_info = PreMoveInfo {
        src: SQ(9),
        dst: SQ(42),
        flags: move_flag,
    };
    let move_info = BitMove::init(pre_move_info);
    assert!(!move_info.is_capture());
    assert!(move_info.is_promo());
    assert_eq!(move_info.promo_piece(), PieceType::Q);
}

fn all_move_flags() -> Vec<MoveFlag> {
    let mut move_flags = Vec::new();
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::P,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::N,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::B,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::R,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::K,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: true,
        prom: PieceType::Q,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::P,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::N,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::B,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::R,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::K,
    });
    move_flags.push(MoveFlag::Promotion {
        capture: false,
        prom: PieceType::Q,
    });
    move_flags.push(MoveFlag::Castle { king_side: true });
    move_flags.push(MoveFlag::Castle { king_side: false });
    move_flags.push(MoveFlag::Capture { ep_capture: true });
    move_flags.push(MoveFlag::Capture { ep_capture: false });
    move_flags.push(MoveFlag::DoublePawnPush);
    move_flags.push(MoveFlag::QuietMove);
    move_flags
}

#[test]
fn sq_from_str() {
    assert_eq!(SQ::from_str("a1").unwrap(), SQ::A1);
    assert_eq!(SQ::from_str("h8").unwrap(), SQ::H8);
    assert_eq!(SQ::from_str("e4").unwrap(), SQ::E4);
    assert_eq!(SQ::from_str("d7").unwrap(), SQ::D7);
    assert!(SQ::from_str("").is_err());
    assert!(SQ::from_str("a").is_err());
    assert!(SQ::from_str("a9").is_err());
    assert!(SQ::from_str("i1").is_err());
    assert!(SQ::from_str("a1b").is_err());
}

#[test]
fn sq_from_str_roundtrip() {
    for i in 0..64u8 {
        let sq = SQ(i);
        let s = sq.to_string();
        let parsed = SQ::from_str(&s).unwrap();
        assert_eq!(sq, parsed);
    }
}

#[test]
fn bitmove_from_str_quiet() {
    let mv = BitMove::from_str("e2e4").unwrap();
    assert_eq!(mv.get_src(), SQ::E2);
    assert_eq!(mv.get_dest(), SQ::E4);
    assert!(!mv.is_promo());
    assert_eq!(mv.to_string(), "e2e4");
}

#[test]
fn bitmove_from_str_promotion() {
    let mv = BitMove::from_str("a7a8q").unwrap();
    assert_eq!(mv.get_src(), SQ::A7);
    assert_eq!(mv.get_dest(), SQ::A8);
    assert!(mv.is_promo());
    assert_eq!(mv.promo_piece(), PieceType::Q);

    let mv = BitMove::from_str("b7b8n").unwrap();
    assert!(mv.is_promo());
    assert_eq!(mv.promo_piece(), PieceType::N);

    let mv = BitMove::from_str("c7c8b").unwrap();
    assert!(mv.is_promo());
    assert_eq!(mv.promo_piece(), PieceType::B);

    let mv = BitMove::from_str("d7d8r").unwrap();
    assert!(mv.is_promo());
    assert_eq!(mv.promo_piece(), PieceType::R);
}

#[test]
fn bitmove_from_str_invalid() {
    assert!(BitMove::from_str("").is_err());
    assert!(BitMove::from_str("e2").is_err());
    assert!(BitMove::from_str("e2e4e6").is_err());
    assert!(BitMove::from_str("e2e9").is_err());
    assert!(BitMove::from_str("e2e4x").is_err());
}

#[test]
fn bitmove_from_str_roundtrip_quiet() {
    let original = BitMove::make_quiet(SQ::E2, SQ::E4);
    let s = original.to_string();
    let parsed = BitMove::from_str(&s).unwrap();
    assert_eq!(parsed.get_src(), original.get_src());
    assert_eq!(parsed.get_dest(), original.get_dest());
}
