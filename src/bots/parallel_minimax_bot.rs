use board::*;
use timer::*;
use piece_move::*;
use engine::Searcher;
use eval::*;
use rayon;
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

    pub fn negate(&mut self) {
        self.score.wrapping_mul(-1);
    }
}

const MAX_PLY: u16 = 5;
const DIVIDE_CUTOFF: usize = 4;

// depth: depth from given
// half_moves: total moves

impl Searcher for ParallelSearcher {

    fn name() -> &'static str {
        "Parallel Searcher"
    }

    fn best_move(mut board: Board, timer: Timer) -> BitMove {
        parallel_minimax(&mut board).best_move.unwrap()
    }

}

fn parallel_minimax(board: &mut Board) -> BestMove {
    if board.depth() == MAX_PLY {
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
        return parallel_task(&moves, board);
    }
}

fn parallel_task(slice: &[BitMove], board: &mut Board) -> BestMove {
    if slice.len() <= DIVIDE_CUTOFF {
        let mut best_value: i16 = NEG_INFINITY;
        let mut best_move: Option<BitMove> = None;
        for mov in slice {
            board.apply_move(*mov);
            let mut returned_move: BestMove = parallel_minimax(board);
            returned_move.negate();
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
            || parallel_task(left, &mut left_clone),
            || parallel_task(right, board));

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
fn bench_parallel(b: &mut Bencher) {
    b.iter(|| {
        let mut b: Board = test::black_box(Board::default());
        let iter = 10;
        (0..50).fold(0, |a: u64, c| {
            let mov = ParallelSearcher::best_move(b.shallow_clone(),timer::Timer::new(20));
            b.apply_move(mov);
            a ^ (b.zobrist()) }
        )
    })
}
