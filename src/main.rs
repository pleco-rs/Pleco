extern crate pleco;
extern crate rand;

use pleco::templates::print_bitboard;
use pleco::engine::*;

use pleco::{board,piece_move,templates,timer};
use pleco::bots::bot_minimax::SimpleBot;
use pleco::bots::bot_random::RandomBot;
use pleco::bots::bot_parallel_minimax::ParallelSearcher;
use pleco::bots::bot_advanced::AdvancedBot;
use pleco::bots::bot_alphabeta::AlphaBetaBot;
use pleco::bots::bot_jamboree::JamboreeSearcher;
use pleco::bots::bot_expert::ExpertBot;



// rnbqkbn1/1ppppppr/7p/p7/7P/4PN2/PPPPQPP1/RNB1KBR b Qq - 1 5
// r2qkbnr/p2ppp2/n5pp/1p2P3/2p2P2/2P3QP/PP1P2P1/RNB1K1NR b kq - 1 12
// r1b1qk1r/pppppp1p/4Nn1b/8/1n1P2PP/8/PPP1PP2/R1BQKB1R w KQ - 1 12
// 3k1b1r/r1p2p2/6p1/2PpQ1qp/pPp3BP/3P4/P2NK1R1/R1B b - - 0 32
// 1r1qkbn1/p2B2pr/b4QP1/1ppp4/P2n1P1p/2P5/1P2P2P/RNB1K1NR b KQ - 2 16

fn main() {
//    gen_random_fens();
    sample_run();
//    compete_multiple(AdvancedBot{}, ExpertBot{}, 60, 10, 5, true);

}

fn test_between() {
    let mut b = board::Board::default();
    let m = b.magic_helper;
    print_bitboard(m.between_bb(24,60));
}

fn sample_run() {
    let max = 400;
    let mut b = board::Board::default();
    let mut i = 0;
    println!("Starting Board");
    b.fancy_print();

    while i < max {
        if b.checkmate() {
            println!("Checkmate");
            i = max;
        } else {
            if i % 57 == 2 {
                let mov = RandomBot::best_move(b.shallow_clone(),&timer::Timer::new(20));
                println!("{}'s move: {}",RandomBot::name(),mov);
                b.apply_move(mov);
            } else if i % 2 == 0 {
                println!("------------------------------------------------");
                println!();
                let mov = AdvancedBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),5);
                println!("{}'s move: {}",AdvancedBot::name(),mov);
                b.apply_move(mov);
            } else {
                let mov = ExpertBot::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 5);
                println!("{}'s move: {}", ExpertBot::name(), mov);
                b.apply_move(mov);
            }
            println!();
            b.fancy_print();
        }
        i += 1;
    }

    b.fancy_print();
}



fn gen_random_fens() {
    let mut b = board::Board::default();
    println!("[");
    println!("\"{}\",",b.get_fen());

    let quota = 4;
    let moves = 0;

    let max = 200;
    let mut i = 0;

    let mut beginning_count = 0;
    let mut middle_count = 0;
    let mut end_count = 0;

    while beginning_count + middle_count + end_count <= (quota * 3) - 1 {
        if i == 0 {
            let mov = RandomBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),1);
            b.apply_move(mov);
            let mov = RandomBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),1);
            b.apply_move(mov);
        }
        if b.checkmate() || i > max {
            if beginning_count + middle_count + end_count > quota * 3 {
                i = max;
            } else {
                i = 0;
                b = board::Board::default();
            }
        } else {
            if i % 11 == 9 {
                let mov = RandomBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),1);
                b.apply_move(mov);
            } else if i % 2 == 0 {
                let mov = JamboreeSearcher::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),5);
                b.apply_move(mov);
            } else {
                let mov = AdvancedBot::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 5);
                b.apply_move(mov);
            }
            i += 1;
        }

        if b.zobrist() % 23 == 11 && b.moves_played() > 7 {
            if b.count_all_pieces() < 13 && end_count < quota {
                println!("\"{}\",",b.get_fen());
                end_count += 1;
            } else if b.count_all_pieces() < 24 && middle_count < quota {
                println!("\"{}\",",b.get_fen());
                middle_count += 1;
            } else if beginning_count < quota {
                println!("\"{}\",",b.get_fen());
                middle_count += 1;
            }
        }
    }

    println!("]");
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