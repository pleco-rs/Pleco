//! Miscellaneous tools for debugging and generating output.
use board::Board;
use engine::Searcher;
use timer::Timer;

use bots::basic::bot_random::RandomBot;
use bots::basic::bot_jamboree::JamboreeSearcher;
use bots::bot_iterative_parallel_mvv_lva::IterativeSearcher;


fn gen_random_fens() {
    let mut b = Board::default();
    println!("[");
    println!("\"{}\",", b.get_fen());

    let quota = 4;

    let max = 200;
    let mut i = 0;

    let beginning_count = 0;
    let mut middle_count = 0;
    let mut end_count = 0;

    while beginning_count + middle_count + end_count <= (quota * 3) - 1 {
        if i == 0 {
            let mov = RandomBot::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 1);
            b.apply_move(mov);
            let mov = RandomBot::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 1);
            b.apply_move(mov);
        }
        if b.checkmate() || i > max {
            if beginning_count + middle_count + end_count > quota * 3 {
                i = max;
            } else {
                i = 0;
                b = Board::default();
            }
        } else {
            if i % 11 == 9 {
                let mov = RandomBot::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 1);
                b.apply_move(mov);
            } else if i % 2 == 0 {
                let mov =
                    JamboreeSearcher::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 5);
                b.apply_move(mov);
            } else {
                let mov = IterativeSearcher::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 5);
                b.apply_move(mov);
            }
            i += 1;
        }

        if b.zobrist() % 23 == 11 && b.moves_played() > 7 {
            if b.count_all_pieces() < 13 && end_count < quota {
                println!("\"{}\",", b.get_fen());
                end_count += 1;
            } else if b.count_all_pieces() < 24 && middle_count < quota {
                println!("\"{}\",", b.get_fen());
                middle_count += 1;
            } else if beginning_count < quota {
                println!("\"{}\",", b.get_fen());
                middle_count += 1;
            }
        }
    }

    println!("]");
}