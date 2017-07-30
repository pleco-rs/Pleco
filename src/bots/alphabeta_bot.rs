use board::*;
use std::cmp::Ordering;
use timer::*;
use piece_move::*;
use engine::Searcher;
use bots::eval::*;
use rayon;
use rayon::prelude::*;
use test::Bencher;
use test;
use timer;

const MAX_PLY: u16 = 5;

pub struct BestMove {
    pub best_move: Option<BitMove>,
    pub score: i16,
}

impl BestMove {
    pub fn new(score: i16) -> Self {
        BestMove{
            best_move: None,
            score: score
        }
    }

    pub fn negate(mut self) -> Self {
        self.score *= -1;
        self
    }
}


pub struct  AlphaBetaBot {
    board: Board,
    timer: Timer,
}

impl Searcher for AlphaBetaBot {
    fn name() -> &'static str {
        "AlphaBeta Searcher"
    }

    fn best_move_depth(mut board: Board, timer: &Timer, max_depth: u16) -> BitMove {
        let alpha: i16 = NEG_INFINITY;
        let beta:  i16 = INFINITY;
        alpha_beta_search(&mut board.shallow_clone(), alpha, beta, max_depth).best_move.unwrap()
    }

    fn best_move(mut board: Board, timer: &Timer) -> BitMove {
        AlphaBetaBot::best_move_depth(board, timer, MAX_PLY)
    }
}

fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {

    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();

    if moves.len() == 0 {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(-STALEMATE);
        }
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
//        board.pretty_print();
//        println!();
//        println!("Applying move: {}", mov);
//        println!("DEEPER ///////////////////////////////////////");
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();
//        println!("UP     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
//        println!("Current Alpha: {}, returned_move_score: {}",alpha,return_move.score);

        if return_move.score > alpha  {
            alpha = return_move.score;
            best_move = Some(mov);
        }

        if alpha >= beta {
            return BestMove{best_move: Some(mov), score: alpha};
        }
    }

    BestMove{best_move: best_move, score: alpha}
}

fn eval_board(board: &mut Board) -> BestMove {
    let m = BestMove::new(Eval::eval_low(&board));
//    println!("score {} at eval", m.score);
//    board.pretty_print();
    m
}

#[test]
pub fn test_fens() {
    use templates::TEST_FENS;
    for str in TEST_FENS.iter() {
        Board::new_from_fen(str);
    }
}


//#[bench]
//fn bench_bot_ply_3__alphabeta_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
////            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),3);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
//
//#[bench]
//fn bench_bot_ply_4__alphabeta_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),4);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
//
//#[bench]
//fn bench_bot_ply_5__alphabeta_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),5);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
//
//
