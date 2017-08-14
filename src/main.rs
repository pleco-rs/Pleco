extern crate pleco;
extern crate rand;

use pleco::templates::print_bitboard;
use pleco::engine::*;
use pleco::timer::Timer;

use pleco::{board,piece_move,templates,timer};

use pleco::bots::basic::bot_minimax::SimpleBot;
use pleco::bots::basic::bot_random::RandomBot;
use pleco::bots::basic::bot_parallel_minimax::ParallelSearcher;
use pleco::bots::basic::bot_alphabeta::AlphaBetaBot;
use pleco::bots::basic::bot_jamboree::JamboreeSearcher;

use pleco::bots::threaded_searcher::ThreadSearcher;
use pleco::bots::bot_expert::ExpertBot;
use pleco::bots::bot_iterative_parallel_mvv_lva::IterativeSearcher;



// rnbqkbn1/1ppppppr/7p/p7/7P/4PN2/PPPPQPP1/RNB1KBR b Qq - 1 5
// r2qkbnr/p2ppp2/n5pp/1p2P3/2p2P2/2P3QP/PP1P2P1/RNB1K1NR b kq - 1 12
// r1b1qk1r/pppppp1p/4Nn1b/8/1n1P2PP/8/PPP1PP2/R1BQKB1R w KQ - 1 12
// 3k1b1r/r1p2p2/6p1/2PpQ1qp/pPp3BP/3P4/P2NK1R1/R1B b - - 0 32
// 1r1qkbn1/p2B2pr/b4QP1/1ppp4/P2n1P1p/2P5/1P2P2P/RNB1K1NR b KQ - 2 16

fn main() {
    let timer = Timer::new(60);
//    gen_random_fens();
//    sample_run();
    compete_multiple(IterativeSearcher{}, ThreadSearcher{timer: &timer},60, 11, 5, true);

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
                let mov = IterativeSearcher::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 5);
                println!("{}'s move: {}", IterativeSearcher::name(), mov);
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
                let mov = IterativeSearcher::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 5);
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
