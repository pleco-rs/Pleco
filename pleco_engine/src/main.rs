extern crate pleco;
extern crate chrono;
extern crate pleco_engine;

use pleco::bot_prelude::{JamboreeSearcher,Searcher,IterativeSearcher};
use pleco::{Board,Player};

use pleco_engine::pleco_searcher::PlecoSearcher;
use pleco_engine::pleco_searcher::misc::PreLimits;

use std::thread;
use chrono::*;


fn main() {
    test_frequency();
}


fn test_frequency() {
    let mut count_frequency: Vec<u64> = vec![0; 500];
    let mut j = 50;
    while j > 0 {
        print!(" {} ...", j);
        let mut s = PlecoSearcher::init(false);

        let mut board = Board::default();
        let mut local: Duration = Duration::seconds(1);
        let max_moves = 500;
        let mut i: usize = max_moves;
        let pleco_side = if j % 2 == 0 { Player::White } else { Player::Black };

        while i > 0 && !board.checkmate() && !board.stalemate() {
            let num_moves = board.moves_played() as usize;
            count_frequency[num_moves] += 1;
            if board.turn() != pleco_side {
                local = Duration::span(|| {
                    let mov = if i < max_moves - 47 {
                        if board.count_all_pieces() < 6 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 8)
                        } else if board.count_all_pieces() < 8 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 7)
                        } else if board.count_all_pieces() < 9 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 6)
                        } else {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 5)
                        }
                    } else {
                        JamboreeSearcher::best_move_depth(board.shallow_clone(), 4)
                    };
                    board.apply_move(mov);
                });
                local = local.max(Duration::milliseconds(1));
            } else {
                s.search(&board, &PreLimits::blank());
                thread::sleep(local.to_std().unwrap());
                let mov = s.stop_search_get_move();
                board.apply_move(mov);
            }
            i -= 1;
        }
        j -= 1;
    }

    println!();
    let mut total_num: u64 = 0;
    for (_num, count) in count_frequency.iter().enumerate() {
        total_num += *count;
    }
    for (num, count) in count_frequency.iter().enumerate() {
        let percent: f64 = (*count as f64 / total_num as f64) * 100.0;
        println!("# Moves: {}, Count: {}, Frequency: {:.4}%, ",num, *count, percent);
    }
}

fn uciloop() {
    let mut s = PlecoSearcher::init(true);
    s.uci();
}

fn run_one() {
    let mut s = PlecoSearcher::init(true);
    let mut board = Board::default();

    let mut local: Duration = Duration::seconds(1);

    let mut i = 0;

    while i < 700 && !board.checkmate() && !board.stalemate() {
        board.pretty_print();

        if i % 2 == 1 {
            local = Duration::span(|| {
                let mov = JamboreeSearcher::best_move_depth(board.shallow_clone(), 4);
                println!("Jamboree searcher: {}", mov);
                board.apply_move(mov);
            });
//            local = local.max(Duration::milliseconds(1));
        } else {
            println!("Pleco Searcher searching for {} micro s", local.num_microseconds().unwrap());
            let start_time = Duration::span(|| { s.search(&board, &PreLimits::blank()); });
            thread::sleep(local.to_std().unwrap());
            println!("Stop!");
            let mov = s.stop_search_get_move();
            println!("Pleco searcher: {}, start_time = {}",mov,start_time.num_microseconds().unwrap());
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
        let mut board = Board::default();

        let mut local: Duration = Duration::seconds(1);

        let max_moves = 250;
        let mut i = max_moves;

        let pleco_side = if j % 2 == 0 {Player::White} else {Player::Black};

        while i > 0 && !board.checkmate() && !board.stalemate() {
            if board.turn() != pleco_side {
                local = Duration::span(|| {
                    let mov = if i < max_moves - 47 {
                        if board.count_all_pieces() < 6 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 8)
                        } else if board.count_all_pieces() < 8 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 7)
                        } else if board.count_all_pieces() < 9 {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 6)
                        } else {
                            JamboreeSearcher::best_move_depth(board.shallow_clone(), 5)
                        }
                    } else {
                        JamboreeSearcher::best_move_depth(board.shallow_clone(), 4)
                    };
                    board.apply_move(mov);
                });
                local = local.max(Duration::milliseconds(1));
            } else {
//                println!("Pleco Searcher searching for {} ms", local.num_milliseconds());
                s.search(&board, &PreLimits::blank());
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
        } else if board.turn() != pleco_side {
            println!("Pleco Wins");
            wins += 1;
        } else {
            println!("Jamboree wins!");
            loses += 1;
        }
        j -= 1;
        println!("rounds = {}, Hash {}",max_moves - i,s.hash_percent());

        if j % 2 == 1 {
            println!("W/L/D {}-{}-{}",wins,loses,draws);
        }
    }
    println!("W/L/D {}-{}-{}",wins,loses,draws);
}