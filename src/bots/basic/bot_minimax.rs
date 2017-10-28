use board::*;
use core::piece_move::*;
use engine::{Searcher,UCILimit};
use board::eval::*;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;


use super::super::BestMove;


const MAX_PLY: u16 = 3;


pub struct SimpleBot {
    board: Board,
}


// depth: depth from given
// half_moves: total moves

impl Searcher for SimpleBot {
    fn name() -> &'static str {
        "Simple Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        minimax(&mut SimpleBot { board: board.shallow_clone()},  max_depth)
            .best_move
            .unwrap()
    }

}

fn minimax(bot: &mut SimpleBot, max_depth: u16) -> BestMove {
    //    println!("depth = {}", bot.board.depth());
    if bot.board.depth() == max_depth {

        return eval_board(bot);
    }

    let moves = bot.board.generate_moves();
    if moves.is_empty() {
        if bot.board.in_check() {
            return BestMove::new(MATE + (bot.board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }
    let mut best_value: i16 = NEG_INFINITY;
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        bot.board.apply_move(mov);
        let returned_move: BestMove = minimax(bot, max_depth).negate();
        bot.board.undo_move();
        if returned_move.score > best_value {
            best_value = returned_move.score;
            best_move = Some(mov);
        }
    }
    BestMove {
        best_move: best_move,
        score: best_value,
    }

}

fn eval_board(bot: &mut SimpleBot) -> BestMove {
    BestMove::new(Eval::eval_low(&bot.board))
}


//#[bench]
//fn bench_bot_ply_3_minimax_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, _c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = SimpleBot::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 3);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist())
//        })
//    })
//}
