use board::*;
use core::*;
use core::piece_move::BitMove;
use engine::*;
use board::eval::*;
use rayon;

#[allow(unused_imports)]
use test::Bencher;
#[allow(unused_imports)]
use test;

const MAX_PLY: u16 = 5;

const DIVIDE_CUTOFF: usize = 6;
const DIVISOR_SEQ: usize = 5;

//                            0   1   2   3   4   5   6   7   8   9
static PLYS_SEQ: [u16; 10] = [0, 1, 2, 2, 2, 2, 2, 3, 3, 3];

pub struct IterativeSearcher {}

impl Searcher for IterativeSearcher {
    fn name() -> &'static str {
        "Advanced Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let max_depth = if limit.is_depth() {limit.depth_limit()} else {MAX_PLY};
        iterative_deepening(board , max_depth)
    }

}


fn iterative_deepening(board: Board, max_depth: u16) -> BitMove {
    // for each level from 1 to max depth, search the node and return the best move and score
    // Once we have reached ply 2, keep the score (say x), c
    //       continue onto previous ply with alpha = x - 33 and beta = x + 33
    //       If ply n + 1 returns with score y && y > x + 33 || y < x - 33
    //          if fail low, redo with alpha = -inf
    //          if fail high, redo with beta = inf
    //       now if re-search fails, do a full ply search with alpha = -inf and beta = inf
    //

    let mut i = 2;
    let mut alpha: i16 = NEG_INFINITY;
    let mut beta: i16 = INFINITY;

    // Create a dummy best_move
    let mut best_move = BestMove::new(NEG_INFINITY);

    // Loop until max_depth is reached
    while i <= max_depth {
        // clone the board
        let mut b = board.shallow_clone();

        let returned_b_move = jamboree(&mut b, alpha, beta, i, PLYS_SEQ[i as usize]);
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
                i += 1;
            }
        }
    }
    if best_move.best_move.is_none() {
        println!("{}, i = {}", best_move.score, i);
    }
    best_move.best_move.unwrap()
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

    let mut moves = board.generate_moves();
    if moves.is_empty() {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        } else {
            return BestMove::new(STALEMATE);
        }
    }

    let amount_seq: usize = 1 + (moves.len() / DIVIDE_CUTOFF) as usize;
    if board.depth() < 5 {
        mvv_lva_sort(&mut moves, &board);
    }

    let (seq, non_seq) = moves.split_at_mut(amount_seq);


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

    if board.depth() >= max_depth {
        if board.in_check() || board.piece_last_captured().is_some() {
            return quiescence_search(board, alpha, beta, max_depth + 2);
        }
        return eval_board(board);
    }

    // Futility Pruning
    if max_depth > 2 && board.depth() == max_depth - 1 &&
        board.piece_last_captured().is_none() && !board.in_check()
    {
        let eval = eval_board(board);
        if eval.score + 100 < alpha {
            return quiescence_search(board, alpha, beta, max_depth + 1);
        }
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

fn quiescence_search(board: &mut Board, mut alpha: i16, beta: i16, max_depth: u16) -> BestMove {
    if board.depth() == max_depth {
        return eval_board(board);
    }

    let moves = if board.in_check() {
        board.generate_moves()
    } else {
        board.generate_moves_of_type(GenTypes::Captures)
    };

    if moves.len() == 0 {
        if board.in_check() {
            return BestMove::new(MATE + (board.depth() as i16));
        }
        return eval_board(board);
    }
    let mut best_move: Option<BitMove> = None;
    for mov in moves {
        board.apply_move(mov);

        let return_move = { quiescence_search(board, -beta, -alpha, max_depth) }.negate();

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

fn q_science_criteria(m: BitMove, _board: &Board) -> bool {
    m.is_capture()
}




fn mvv_lva_sort(moves: &mut [BitMove], board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture() {
            board.captured_piece(*a).unwrap().value() - piece.value()
        } else if piece == Piece::P {
            if a.is_double_push().0 {
                -2
            } else {
                -3
            }
        } else {
            -4
        }
    })
}





fn eval_board(board: &mut Board) -> BestMove {
    BestMove::new(Eval::eval_low(&board))
}



pub struct BestMove {
    pub best_move: Option<BitMove>,
    pub score: i16,
}

impl BestMove {
    pub fn new(score: i16) -> Self {
        BestMove {
            best_move: None,
            score: score,
        }
    }

    pub fn negate(mut self) -> Self {
        self.score *= -1;
        self
    }
}



//
//#[bench]
//fn bench_bot_ply_3_main_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = IterativeSearcher::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 3);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}

//#[bench]
//fn bench_bot_ply_4_main_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, _c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = IterativeSearcher::best_move_depth(b.shallow_clone(), &Timer::new_no_inc(20), 4);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist())
//        })
//    })
//}


//#[bench]
//fn bench_bot_ply_5_main_bot(b: &mut Bencher) {
//    use templates::TEST_FENS;
//    b.iter(|| {
//        let mut b: Board = test::black_box(Board::default());
//        let iter = TEST_FENS.len();
//        let mut i = 0;
//        (0..iter).fold(0, |a: u64, c| {
//            //            println!("{}",TEST_FENS[i]);
//            let mut b: Board = test::black_box(Board::new_from_fen(TEST_FENS[i]));
//            let mov = AdvancedBot::best_move_depth(b.shallow_clone(), &timer::Timer::new(20), 5);
//            b.apply_move(mov);
//            i += 1;
//            a ^ (b.zobrist()) }
//        )
//    })
//}
