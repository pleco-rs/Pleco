//! Miscellaneous tools for debugging and generating output.
use board::Board;
use engine::Searcher;
use timer::Timer;
use rand;

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
// "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
// https://chess.stackexchange.com/questions/1482/how-to-know-when-a-fen-position-is-legal
// TODO: Finish
pub fn is_valid_fen(fen: &str) -> bool {
    // split the string by white space
    let det_split: Vec<&str> = fen.split_whitespace().collect();

    // must have 6 parts :
    // [ Piece Placement, Side to Move, Castling Ability, En Passant square, Half moves, full moves]
    if det_split.len() != 6 { return false; }

    // Split the first part by '/' for locations
    let b_rep: Vec<&str> = det_split[0].split('/').collect();

    // 8 ranks, so 8 parts
    if b_rep.len() != 8 { return false; }





    unimplemented!();
}

/// Generates a board with a Random Position
pub fn gen_rand_legal_board() -> Board {
    loop {
        let mut board = Board::default();
        let mut i = 0;
        let mut moves = board.generate_moves();
        while i < 70 && !moves.is_empty() {
            if i > 3 {
                if rand::random::<i32>() % (71 - i) == 0  || rand::random::<usize>() % 219 == 0 {
                    return board;
                }
            }
            // apply random move
            if rand::random::<usize>() % 10 == 0 {
                RandomBot::best_move_depth(board.shallow_clone(), &Timer::new_no_inc(20), 1);
            } else if rand::random::<usize>() % 4 == 0 {
                let best_move = JamboreeSearcher::best_move_depth(board.parallel_clone(),&Timer::new_no_inc(30),5);
                board.apply_move(best_move);
            } else if rand::random::<usize>() % 3 == 0 {
                let best_move = IterativeSearcher::best_move_depth(board.parallel_clone(),&Timer::new_no_inc(30),4);
                board.apply_move(best_move);
            } else {
                let best_move = JamboreeSearcher::best_move_depth(board.parallel_clone(),&Timer::new_no_inc(30),4);
                board.apply_move(best_move);
            }

            moves = board.generate_moves();
            i += 1;
        }
    }
}