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
        self.score.wrapping_neg();
        self
    }
}


pub struct AlphaBetaBot {
    board: Board,
    timer: Timer,
}

impl Searcher for AlphaBetaBot {
    fn name() -> &'static str {
        "Random Searcher"
    }

    fn best_move_depth(mut board: Board, timer: Timer, max_depth: u16) -> BitMove {
        let mut bot = AlphaBetaBot { board: board, timer: timer};
        let alpha: i16 = NEG_INFINITY;
        let beta:  i16 = INFINITY;
        alpha_beta_search(&mut bot.board.parallel_clone(), alpha, beta, max_depth).best_move.unwrap()
    }

    fn best_move(mut board: Board, timer: Timer) -> BitMove {
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
            return BestMove::new(NEG_INFINITY - (board.depth() as i16));
        } else {
            return BestMove::new(-STALEMATE);
        }
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();

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
    BestMove::new(Eval::eval(&board))
}

#[bench]
fn bench_bot_ply_4__alphabeta_bot(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 2;
        (0..iter).fold(0, |a: u64, c| {
            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),timer::Timer::new(20),4);
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}

#[bench]
fn bench_bot_ply_5__alphabeta_bot(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 2;
        (0..iter).fold(0, |a: u64, c| {
            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),timer::Timer::new(20),5);
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}

#[bench]
fn bench_bot_ply_6__alphabeta_bot(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 2;
        (0..iter).fold(0, |a: u64, c| {
            let mov = AlphaBetaBot::best_move_depth(b.shallow_clone(),timer::Timer::new(20),6);
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}


