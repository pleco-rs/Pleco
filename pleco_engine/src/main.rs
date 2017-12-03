extern crate pleco;
extern crate chrono;
extern crate pleco_engine;

use pleco::bot_prelude::{JamboreeSearcher,Searcher,IterativeSearcher};
use pleco::Board;
use pleco::tools::UCILimit;

use pleco_engine::pleco_searcher::PlecoSearcher;

use std::thread;
use chrono::*;


fn main() {
    run_many();
}

fn uciloop() {
    let mut s = PlecoSearcher::init(true);
    s.uci();
}

fn run_one() {
    let mut s = PlecoSearcher::init(true);
    let mut board = Board::default();

    let mut local: Duration = Duration::seconds(2);

    let mut i = 0;

    while i < 700 && !board.checkmate() && !board.stalemate() {
        board.pretty_print();

        if i % 2 == 1 {
            local = Duration::span(|| {
                let mov = JamboreeSearcher::best_move_depth(board.shallow_clone(), 5);
                println!("Jamboree searcher: {}", mov);
                board.apply_move(mov);
            });
        } else {
            s.search(&board, &UCILimit::Infinite);
            thread::sleep_ms(local.num_milliseconds() as u32);
            println!("Stop!");
            let mov = s.stop_search_get_move();
            println!("Pleco searcher: {}",mov);
            board.apply_move(mov);
        }
        i += 1;
    }


    println!("i = {}",i);
    board.pretty_print();
    if i % 2 == 1 {
        println!("Pleco Wins");
    } else {
        println!("Jamboree wins!");
    }

    println!("i = {}",i);
    board.pretty_print();
    if i % 2 == 1 {
        println!("Pleco Wins");
    } else {
        println!("Jamboree wins!");
    }

}

fn run_many() {
    let mut j = 1000;
    let mut wins = 0;
    let mut loses = 0;
    let mut draws = 0;
    while j > 0 {
        let mut s = PlecoSearcher::init(false);
        s.use_stdout(false);
        s.clear_tt();
        let mut board = Board::default();

        let mut local: Duration = Duration::seconds(1);

        let max_moves = 250;
        let mut i = max_moves;

        while i > 0 && !board.checkmate() && !board.stalemate() {
            if i % 2 == 1 {
                local = Duration::span(|| {
                    let mov = if i < max_moves - 40 {
                        JamboreeSearcher::best_move_depth(board.shallow_clone(), 5)
                    } else {
                        JamboreeSearcher::best_move_depth(board.shallow_clone(), 5)
                    };
                    board.apply_move(mov);
                });
                local = local.max(Duration::milliseconds(1));
            } else {
                s.search(&board, &UCILimit::Infinite);
                thread::sleep(local.to_std().unwrap());
                let mov = s.stop_search_get_move();
                board.apply_move(mov);
            }
            i -= 1;
        }

        board.pretty_print();
        if board.stalemate() || i == 0 || !board.in_check() {
            print!("Draw! rem: {} ", i % 2);
            if i == 0 {
                println!("i == 0");
            } else if board.stalemate() {
                println!("Stalemate");
            } else {
                println!("Not in check");
            }
            draws += 1;
        } else if i % 2 == 1 {
            println!("Pleco Wins");
            wins += 1;
        } else {
            println!("Jamboree wins!");
            loses += 1;
        }
        j -= 1;
        println!("rounds = {}, Hash {}",max_moves - i,s.hash_percent());

        if j % 3 == 2 {
            println!("W/L/D {}-{}-{}",wins,loses,draws);
        }
    }
    println!("W/L/D {}-{}-{}",wins,loses,draws);
}