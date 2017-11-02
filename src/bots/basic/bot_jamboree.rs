use board::*;
use timer::*;
use core::piece_move::*;
use engine::{Searcher,UCILimit};
use board::eval::*;
use rayon;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;


use super::super::BestMove;



pub struct JamboreeSearcher {
    board: Board,
    timer: Timer,
}


const MAX_PLY: u16 = 5;

const DIVIDE_CUTOFF: usize = 5;
const DIVISOR_SEQ: usize = 4;

// depth: depth from given
// half_moves: total moves

impl Searcher for JamboreeSearcher {
    fn name() -> &'static str {
        "Jamboree Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        let alpha = NEG_INFINITY;
        let beta = INFINITY;
        jamboree(&mut board.shallow_clone(), alpha, beta, max_depth, 2)
            .best_move
            .unwrap()
    }
}

fn jamboree(
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    max_depth: u16,
    plys_seq: u16,
) -> BestMove {
    assert!(alpha <= beta);
    if board.depth() >= max_depth {
        return eval_board(board);
    }

    if board.depth() >= max_depth - plys_seq {
        return alpha_beta_search(board, alpha, beta, max_depth);
    }

    let moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVIDE_CUTOFF) as usize;
    let (seq, non_seq) = moves.split_at(amount_seq);

    let mut best_move: Option<BitMove> = None;
    for mov in seq {
        board.apply_move(*mov);
        let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
        board.undo_move();

        if return_move.score > alpha {
            alpha = return_move.score;
            best_move = Some(*mov);
        }

        if alpha >= beta {
            return BestMove {
                best_move: Some(*mov),
                score: alpha,
            };
        }
    }

    let returned_move = parallel_task(non_seq, board, alpha, beta, max_depth, plys_seq);

    if returned_move.score > alpha {
        returned_move
    } else {
        BestMove {
            best_move: best_move,
            score: alpha,
        }
    }
}

fn parallel_task(
    slice: &[BitMove],
    board: &mut Board,
    mut alpha: i16,
    beta: i16,
    max_depth: u16,
    plys_seq: u16,
) -> BestMove {
    let mut best_move: Option<BitMove> = None;
    if slice.len() <= DIVIDE_CUTOFF {
        for mov in slice {
            board.apply_move(*mov);
            let return_move = jamboree(board, -beta, -alpha, max_depth, plys_seq).negate();
            board.undo_move();

            if return_move.score > alpha {
                alpha = return_move.score;
                best_move = Some(*mov);
            }

            if alpha >= beta {
                return BestMove {
                    best_move: Some(*mov),
                    score: alpha,
                };
            }
        }

    } else {
        let mid_point = slice.len() / 2;
        let (left, right) = slice.split_at(mid_point);
        let mut left_clone = board.parallel_clone();

        let (left_move, right_move) = rayon::join (
            || parallel_task(left, &mut left_clone, alpha, beta, max_depth, plys_seq),
            || parallel_task(right, board, alpha, beta, max_depth, plys_seq));

        if left_move.score > alpha {
            alpha = left_move.score;
            best_move = left_move.best_move;
        }
        if right_move.score > alpha {
            alpha = right_move.score;
            best_move = right_move.best_move;
        }
    }
    BestMove {
        best_move: best_move,
        score: alpha,
    }

}



fn alpha_beta_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = board.generate_moves();

    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(-STALEMATE);
        }
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);
        let return_move = alpha_beta_search(board, -beta, -alpha, max_depth).negate();
        board.undo_move();

        if return_move.score > alpha {
            alpha = return_move.score;
            best_move = Some(mov);
        }

        if alpha >= beta {
            return BestMove {
                best_move: Some(mov),
                score: alpha,
            };
        }
    }

    BestMove {
        best_move: best_move,
        score: alpha,
    }
}

fn eval_board(board: &mut Board) -> BestMove {
    BestMove::new(Eval::eval_low(board))
}



//
//
//#[bench]
//fn bench_bot_ply_3__jamboree_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = JamboreeSearcher::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),3);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
//
//#[bench]
//fn bench_bot_ply_4__jamboree_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = JamboreeSearcher::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),4);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}

//
//#[bench]
//fn bench_bot_ply_5__jamboree_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = JamboreeSearcher::best_move_depth(b.shallow_clone(),&timer::Timer::new(20),5);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
