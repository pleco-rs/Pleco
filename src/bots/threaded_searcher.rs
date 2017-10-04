use rayon;

use board::Board;
use std::sync::atomic::*;
use engine::Searcher;
use transposition_table::*;
use eval::*;
use piece_move::BitMove;
use timer::Timer;
use templates::*;

use super::BestMove;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

lazy_static!(
    static ref NODES_VISITED: AtomicIsize = AtomicIsize::new(0);
    static ref TABLE: TranspositionTable = TranspositionTable::with_capacity(TABLE_INIT_SPACE);
    static ref CONTINUE: AtomicBool = AtomicBool::new(true);
//    static ref CONF: Configuration  = Configuration::default();
);

static mut CONF: Configuration =  Configuration {
    divide_cutoff: DEFAULT_DIVIDE_CUTOFF,
    ratio_parallel: DEFAULT_RATIO_PARALLEL,
    max_ply: DEFAULT_MAX_PLY,
    ply_seq: DEFAULT_PLY_SEQ,
    q_sci_depth:DEFAULT_Q_SCI_DEPTH,
};

const TABLE_INIT_SPACE: usize = 40000;
const ORDER: Ordering = Ordering::Relaxed;

//
//struct Configuration {
//    pub divide_cutoff: AtomicUsize,
//    pub ratio_parallel: AtomicUsize,
//    pub max_ply: AtomicU16,
//    pub ply_seq: AtomicU16,
//    pub q_sci_depth: AtomicU16,
//}

struct Configuration {
    pub divide_cutoff: usize,
    pub ratio_parallel: usize,
    pub max_ply: u16,
    pub ply_seq: u16,
    pub q_sci_depth: u16,
}



const DEFAULT_DIVIDE_CUTOFF: usize = 6;
const DEFAULT_RATIO_PARALLEL: usize = 5;
const DEFAULT_MAX_PLY: u16 = 5;
const DEFAULT_PLY_SEQ: u16 = 2;
const DEFAULT_Q_SCI_DEPTH: u16 = 7;


//impl Default for Configuration {
//    fn default() -> Self {
//        Configuration {
//            divide_cutoff: AtomicUsize::new(DEFAULT_DIVIDE_CUTOFF),
//            ratio_parallel: AtomicUsize::new(DEFAULT_RATIO_PARALLEL),
//            max_ply: AtomicU16::new(DEFAULT_MAX_PLY),
//            ply_seq: AtomicU16::new(DEFAULT_PLY_SEQ),
//            q_sci_depth: AtomicU16::new(DEFAULT_Q_SCI_DEPTH),
//        }
//    }
//}
//
//impl Configuration {
//    pub fn set_plys(&mut self, ply: u16) {
//        CONF.max_ply.store(ply, ORDER);
//        CONF.ply_seq.store(PLYS_SEQ[ply as usize], ORDER);
//    }
//}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            divide_cutoff: DEFAULT_DIVIDE_CUTOFF,
            ratio_parallel: DEFAULT_RATIO_PARALLEL,
            max_ply: DEFAULT_MAX_PLY,
            ply_seq: DEFAULT_PLY_SEQ,
            q_sci_depth:DEFAULT_Q_SCI_DEPTH,
        }
    }
}

impl Configuration {
    pub fn set_plys(&mut self, ply: u16) {
        self.max_ply = ply;
        self.ply_seq = PLYS_SEQ[ply as usize];
    }
}

//                             0   1   2   3   4   5   6   7   8   9
static PLYS_SEQ: [u16; 10] =  [0 , 1,  2,  2,  2,  2,  2,  3,  3,  3];


pub struct ThreadSearcher<'a> {
    pub timer: &'a Timer
}

impl <'a> Searcher for ThreadSearcher<'a> {
    fn name() -> &'static str {
        "Thread Searcher"
    }

    fn best_move_depth(mut board: Board, timer: &Timer, max_depth: u16) -> BitMove {
        let searcher: ThreadSearcher = ThreadSearcher {timer};
        reset_global_state(false);
        reset_ply(max_depth);
        searcher.iterative_deepening(&mut board)
    }

    fn best_move(_board: Board, _timer: &Timer) -> BitMove {
        unimplemented!()
    }
}

impl <'a> ThreadSearcher <'a> {
    fn iterative_deepening(&self, board: &mut Board) -> BitMove {
        let max_ply = max_ply();
        let mut i = 2;
        let mut alpha: i16 = NEG_INFINITY;
        let mut beta:  i16 = INFINITY;
        let mut best_move = BestMove::new(NEG_INFINITY);

        while i <= max_ply {
            let mut b = board.shallow_clone();
            reset_ply(i);

            let returned_b_move = search(&mut b, alpha, beta);
            if i >= 2 {
                if returned_b_move.score > beta {
                    beta = INFINITY;
                } else if returned_b_move.score < alpha {
                    alpha = NEG_INFINITY;
                } else {
                    if returned_b_move.best_move.is_some() {
                        alpha = returned_b_move.score - 34;
                        beta = returned_b_move.score + 34;
                        best_move = returned_b_move;
                    }

                }
            }
            i += 1;
        }
        best_move.best_move.unwrap()
    }
}

fn search(board: &mut Board, mut alpha: i16, beta: i16) -> BestMove {
    let is_seq: bool = board.depth() >= max_ply() - ply_seq();

    if board.depth() >= max_ply() {
        if board.in_check() || board.piece_last_captured().is_some() {
            return quiescence_search(board, alpha, beta)
        }
        return eval_board(board);
    }

    if max_ply() > 2 && board.depth() == max_ply() - 1 && board.piece_last_captured().is_none() && !board.in_check() {
        let eval = eval_board(board);
        if eval.score + 100 < alpha {
            return quiescence_search(board, alpha, beta);
        }
    }

    let mut moves: Vec<BitMove> = board.generate_moves();

    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }

    let amount_seq: usize = if is_seq { moves.len() }
        else {  1 + (moves.len() / divide_cutoff()) as usize  };

    if board.depth() < 5 { mvv_lva_sort(&mut moves, &board); }

    let (seq, non_seq) = moves.split_at_mut(amount_seq);

    let mut best_move: Option<BitMove> = None;
    for mov in seq {
        board.apply_move(*mov);
        let return_move = search(board, -beta, -alpha).negate();
        board.undo_move();

        eval_alpha(&mut alpha, return_move.score, &mov, &mut best_move);

        if alpha >= beta {
            return BestMove{best_move: Some(*mov), score: alpha};
        }
    }


    if !non_seq.is_empty() {
        let returned_move = parallel_task(non_seq, board, alpha, beta);
        if returned_move.score > alpha {
             return returned_move;
        }
    }
    BestMove{best_move: best_move, score: alpha}
}


fn parallel_task(slice: &[BitMove], board: &mut Board, mut alpha: i16, beta: i16) -> BestMove {
    let mut best_move: Option<BitMove> = None;
    if slice.len() <= divide_cutoff() {
        for mov in slice {
            board.apply_move(*mov);
            let return_move = search(board, -beta, -alpha).negate();
            board.undo_move();

            eval_alpha(&mut alpha, return_move.score, &mov, &mut best_move);

            if alpha >= beta {
                return BestMove{best_move: Some(*mov), score: alpha};
            }
        }

    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join (
            || parallel_task(left, &mut left_clone, alpha, beta),
            || parallel_task(right, board, alpha, beta));

        if left_move.score > alpha {
            alpha = left_move.score;
            best_move = left_move.best_move;
        }
        if right_move.score > alpha {
            alpha = right_move.score;
            best_move = right_move.best_move;
        }
    }

    BestMove{best_move: best_move, score: alpha}
}


fn quiescence_search(board: &mut Board, mut alpha: i16, beta: i16) -> BestMove {
    if board.depth() >= q_sci_depth() { return eval_board(&board); }

    let moves = if board.in_check() {
        board.generate_moves()
    } else {
        board.generate_moves_of_type(GenTypes::Captures)
    };

    if moves.is_empty() {
        return empty_moves(&board, true);
    }

    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
        let return_move = quiescence_search(board, -beta, -alpha).negate();
        board.undo_move();

        eval_alpha(&mut alpha, return_move.score, &mov, &mut best_move);

        if alpha >= beta { return BestMove{best_move: Some(mov), score: alpha}; }
    }

    BestMove{best_move: best_move, score: alpha}
}

fn mvv_lva_sort(moves: &mut[BitMove], board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture()  {
            value_of_piece(board.captured_piece(*a).unwrap())
                - value_of_piece(piece)
        } else if piece == Piece::P {
            if a.is_double_push().0 {-2}
                else {-3}
        } else {
            -4
        }
    })
}

#[inline]
fn eval_board(board: &Board) -> BestMove {
    BestMove::new(Eval::eval_low(&board))
}

#[inline]
fn empty_moves(board: &Board, eval: bool) -> BestMove {
    if board.in_check() {
        return BestMove::new(MATE + (board.depth() as i16));
    } else if eval {
        return eval_board(board);
    }
    return BestMove::new(STALEMATE);
}

#[inline(always)]
fn eval_alpha(alpha: &mut i16, score: i16, curr_move: &BitMove, best_move: &mut Option<BitMove>) {
    if score > *alpha {
        *alpha = score;
        *best_move = Some(*curr_move);
    }
}

//#[inline(always)]
//fn reset_global_state(clear_tt: bool) {
//    NODES_VISITED.store(0, ORDER);
//    CONTINUE.store(true, ORDER);
//    reset_conf();
//    if clear_tt {
//        TABLE.clear();
//        TABLE.reserve(TABLE_INIT_SPACE);
//    }
//}
//
//#[inline(always)]
//fn reset_ply(ply: u16) {
//    CONF.max_ply.store(ply, ORDER);
//    CONF.ply_seq.store(PLYS_SEQ[ply as usize], ORDER);
//}
//
//#[inline(always)]
//fn reset_conf() {
//    CONF.divide_cutoff.store(DEFAULT_DIVIDE_CUTOFF, ORDER);
//    CONF.ratio_parallel.store(DEFAULT_RATIO_PARALLEL, ORDER);
//    CONF.max_ply.store(DEFAULT_MAX_PLY, ORDER);
//    CONF.ply_seq.store(DEFAULT_PLY_SEQ, ORDER);
//    CONF.q_sci_depth.store(DEFAULT_Q_SCI_DEPTH, ORDER);
//}
//
//#[inline(always)]
//fn divide_cutoff() -> usize {
//    CONF.divide_cutoff.load(ORDER)
//}
//
//#[inline(always)]
//fn ratio_parallel() -> usize {
//    CONF.ratio_parallel.load(ORDER)
//}
//
//#[inline(always)]
//fn max_ply() -> u16 {
//    CONF.max_ply.load(ORDER)
//}
//
//#[inline(always)]
//fn ply_seq() -> u16 {
//    CONF.ply_seq.load(ORDER)
//}
//
//#[inline(always)]
//fn q_sci_depth() -> u16 {
//    CONF.q_sci_depth.load(ORDER)
//}


#[inline(always)]
fn reset_global_state(clear_tt: bool) {
    NODES_VISITED.store(0, ORDER);
    CONTINUE.store(true, ORDER);
    reset_conf();
    if clear_tt {
        TABLE.clear();
        TABLE.reserve(TABLE_INIT_SPACE);
    }
}

#[inline(always)]
fn reset_ply(ply: u16) {
    unsafe { CONF.set_plys(ply) };
}

#[inline(always)]
fn reset_conf() {
    unsafe { CONF = Configuration::default() };
}

#[inline(always)]
fn divide_cutoff() -> usize {
    unsafe { CONF.divide_cutoff}
}

#[inline(always)]
fn ratio_parallel() -> usize {
    unsafe { CONF.ratio_parallel}
}

#[inline(always)]
fn max_ply() -> u16 {
    unsafe { CONF.max_ply}
}

#[inline(always)]
fn ply_seq() -> u16 {
    unsafe { CONF.ply_seq}
}

#[inline(always)]
fn q_sci_depth() -> u16 {
    unsafe { CONF.q_sci_depth }
}

//#[bench]
//fn bench_bot_ply_3_threaded_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    use test;
//    b.iter(|| {
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, _c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = ThreadSearcher::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 3);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
//
//#[bench]
//fn bench_bot_ply_4_threaded_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    use test;
//    b.iter(|| {
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, _c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = ThreadSearcher::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 4);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}