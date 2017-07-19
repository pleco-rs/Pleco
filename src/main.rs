extern crate rusty_chess;
extern crate rand;
use rusty_chess::{board,piece_move,templates};





fn main() {
    let mut b = board::Board::default();
    let mut i = 0;
    while i < 100 {
        b.fancy_print();
        println!("{}",b.get_fen());
        let moves = b.generate_moves();
        let len = moves.len();
        let mut x: usize = rand::random::<usize>();
        if x >= len {
            x = rand::random::<usize>() % len;
        }
        println!("{}",moves[x]);
        println!();

        b.apply_move(moves[x]);
        i += 1;
    }

    b.fancy_print();


}


fn test_moving() {
    let mut b = board::Board::default();
    let p = piece_move::PreMoveInfo {
        src: 12,
        dst: 28,
        flags: piece_move::MoveFlag::DoublePawnPush
    };
    let m = piece_move::BitMove::init(p);
    b.fancy_print();
    b.apply_move(m);
    b.fancy_print();
    let p = piece_move::PreMoveInfo {
        src: 51,
        dst: 35,
        flags: piece_move::MoveFlag::DoublePawnPush
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();
    let p = piece_move::PreMoveInfo {
        src: 28,
        dst: 35,
        flags: piece_move::MoveFlag::Capture {ep_capture: false}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();
    //
    //    templates::print_bitboard(b.get_occupied_player(templates::Player::White));
    //    println!("");
    //    templates::print_bitboard(b.get_occupied_player(templates::Player::Black));
    //    templates::print_bitboard(b.get_occupied());
    let p = piece_move::PreMoveInfo {
        src: 59,
        dst: 35,
        flags: piece_move::MoveFlag::Capture {ep_capture: false}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: 5,
        dst: 12,
        flags: piece_move::MoveFlag::QuietMove,
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: 35,
        dst: 8,
        flags: piece_move::MoveFlag::Capture {ep_capture: false}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: 6,
        dst: 21,
        flags: piece_move::MoveFlag::QuietMove
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: 60,
        dst: 59,
        flags: piece_move::MoveFlag::QuietMove
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: 4,
        dst: 7,
        flags: piece_move::MoveFlag::Castle{king_side: true}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: templates::Square::A2 as u8,
        dst: templates::Square::B1 as u8,
        flags: piece_move::MoveFlag::Capture {ep_capture: false}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();

    let p = piece_move::PreMoveInfo {
        src: templates::Square::A2 as u8,
        dst: templates::Square::B1 as u8,
        flags: piece_move::MoveFlag::Capture {ep_capture: false}
    };
    let m = piece_move::BitMove::init(p);
    b.apply_move(m);
    b.fancy_print();





    let moves = b.generate_moves();

    for x in moves.iter() {
        println!("{}",x)
    }

    println!("{}",moves.len());

}