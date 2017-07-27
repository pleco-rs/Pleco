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




pub struct ParallelSearcher {
    board: Board,
    timer: Timer,
}

pub struct BestMove {
    best_move: Option<BitMove>,
    score: i16,
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

    pub fn score(&self) -> i16 {self.score}
}




const MAX_PLY: u16 = 5;
const DIVIDE_CUTOFF: usize = 8;

// depth: depth from given
// half_moves: total moves

impl Searcher for ParallelSearcher {

    fn name() -> &'static str {
        "Parallel Searcher"
    }

    fn best_move(mut board: Board, timer: Timer) -> BitMove {
        ParallelSearcher::best_move_depth(board, timer, MAX_PLY)
    }

    fn best_move_depth(mut board: Board, timer: Timer, max_depth: u16) -> BitMove {
        parallel_minimax(&mut board, max_depth).best_move.unwrap()
    }


}

fn parallel_minimax(board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();
    if moves.len() == 0 {
        if board.in_check() {
            return BestMove::new(NEG_INFINITY - (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    } else {
        return parallel_task(&moves, board, max_depth);
    }
}

fn parallel_task(slice: &[BitMove], board: &mut Board, max_depth: u16) -> BestMove {
    if board.depth() == max_depth - 2 ||  slice.len() <= DIVIDE_CUTOFF {
        let mut best_value: i16 = NEG_INFINITY;
        let mut best_move: Option<BitMove> = None;
        for mov in slice {
            board.apply_move(*mov);
            let mut returned_move: BestMove = parallel_minimax(board, max_depth).negate();
            board.undo_move();
            if returned_move.score > best_value {
                best_value = returned_move.score;
                best_move = Some(*mov);
            }
        }
        return BestMove{best_move: best_move, score: best_value};
    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join(
            || parallel_task(left, &mut left_clone, max_depth),
            || parallel_task(right, board, max_depth));

        if left_move.score > right_move.score {
            return left_move;
        } else {
            return right_move;
        }
    }
}

fn eval_board(board: &mut Board) -> BestMove {
    BestMove::new(Eval::eval(&board))
}


#[bench]
fn bench_bot_ply_4__parallel_bot(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 2;
        (0..iter).fold(0, |a: u64, c| {
            let mov = ParallelSearcher::best_move_depth(b.shallow_clone(),timer::Timer::new(20),4);
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}
