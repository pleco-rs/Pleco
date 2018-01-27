//! The main searching function.
use time::uci_timer::*;

use std::cmp::{min,max};
use std::sync::atomic::{Ordering,AtomicBool};
use std::sync::Arc;

use pleco::{MoveList,Board,BitMove};
use pleco::core::*;
use pleco::tools::tt::*;
use pleco::core::score::*;
use pleco::tools::eval::Eval;

use super::misc::*;
use MAX_PLY;
use TT_TABLE;

use time::time_management::TimeManager;
use super::threads::TIMER;
use super::root_moves::root_moves_list::RootMoveList;
use consts::*;

const THREAD_DIST: usize = 20;
//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [u16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [u16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];


pub struct ThreadSearcher {
    pub limit: Limits,
    pub board: Board,
    pub time_man: &'static TimeManager,
    pub tt: &'static TranspositionTable,
    pub thread_stack: [ThreadStack; THREAD_STACK_SIZE],
    pub id: usize,
    pub root_moves: RootMoveList,
    pub use_stdout: Arc<AtomicBool>,
}

impl ThreadSearcher {
    pub fn search_root(&mut self) {
        assert!(self.board.depth() == 0);
        self.root_moves.set_finished(false);
        if self.stop() {
            self.root_moves.set_finished(true);
            return;
        }

        if self.use_stdout() {
            println!("info id {} start", self.id);
        }

//        if self.main_thread() {
//            println!("info max_time: {}, ideal time: {}", self.time_man.maximum_time(), self.time_man.ideal_time());
//        }

        let max_depth = if let LimitsType::Depth(d) = self.limit.limits_type {
            d
        } else {
            MAX_PLY
        };

        let start_ply: u16 = START_PLY[self.id % THREAD_DIST];
        let skip_size: u16 = SKIP_SIZE[self.id % THREAD_DIST];
        let mut depth: u16 = start_ply;

        let mut delta: i32 = Value::NEG_INFINITE.0 as i32;
        #[allow(unused_assignments)]
        let mut best_value: i32 = Value::NEG_INFINITE.0 as i32;
        let mut alpha: i32 = Value::NEG_INFINITE.0 as i32;
        let mut beta: i32 = Value::INFINITE.0 as i32;

        let mut time_reduction: f64 = 1.0;
        let mut last_best_move: BitMove = BitMove::null();
        let mut best_move_stability: u32 = 0;

        self.root_moves.shuffle(self.id, &self.board);


        'iterative_deepening: while !self.stop() && depth < max_depth {
            self.root_moves.rollback();

            if depth >= 5 {
                delta = 18;
                alpha = max(self.root_moves.prev_best_score() - delta, Value::NEG_INFINITE.0 as i32);
                beta = min(self.root_moves.prev_best_score() + delta, Value::INFINITE.0 as i32);
            }

            'aspiration_window: loop {

                best_value = self.search::<PV>(alpha, beta, depth) as i32;
                self.root_moves.sort();

                if self.stop() {
                    break 'aspiration_window;
                }

                if best_value <= alpha {
                    alpha = max(best_value - delta, Value::NEG_INFINITE.0 as i32);
                } else if best_value >= beta {
                    beta = min(best_value + delta, Value::INFINITE.0 as i32);
                } else {
                    break 'aspiration_window;
                }
                delta += (delta / 4) + 5;

                assert!(alpha >= Value::NEG_INFINITE.0 as i32);
                assert!(beta <= Value::INFINITE.0 as i32);
            }

            self.root_moves.sort();
            if self.use_stdout() && self.main_thread() {
                println!("info depth {}", depth);
            }
            if !self.stop() {
                self.root_moves.set_depth_completed(depth);
            }
            depth += skip_size;

            if !self.main_thread() {
                continue;
            }

            let best_move = self.root_moves.first().bit_move;
            if best_move != last_best_move {
                time_reduction = 1.0;
                best_move_stability = 0;
            } else {
                time_reduction *= 0.91;
                best_move_stability += 1;
            }

            last_best_move = best_move;

            // check for time
            if let Some(_) = self.limit.use_time_management() {
                if !self.stop() {
//                    let prev_best = self.thread.root_moves.first().prev_score;
                    let ideal = TIMER.ideal_time();
                    let elapsed = TIMER.elapsed();
                    let stability: f64 = f64::powi(0.92, best_move_stability as i32);
                    let new_ideal = (ideal as f64 * stability * time_reduction) as i64;
                    println!("ideal: {}, new_ideal: {}, elapsed: {}", ideal, new_ideal, elapsed);
                    if self.root_moves.len() == 1 || TIMER.elapsed() >= new_ideal {
                        break 'iterative_deepening;
                    }
                }
            }

        }
        self.root_moves.set_finished(true);
    }

    fn search<N: PVNode>(&mut self, mut alpha: i32, beta: i32, max_depth: u16) -> i32 {
        let is_pv: bool = N::is_pv();
        let at_root: bool = self.board.depth() == 0;
        let zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value = if tt_hit {tt_entry.score as i32} else {0};
        let in_check: bool = self.board.in_check();
        let ply = self.board.depth();

        let mut best_move = BitMove::null();

        let mut value = Value::NEG_INFINITE.0 as i32;
        let mut best_value = Value::NEG_INFINITE.0 as i32;
        let mut moves_played = 0;

        let mut pos_eval: i32 = 0;

        if self.main_thread() {
            self.check_time();
        }

        if ply >= max_depth || self.stop() {
            return Eval::eval_low(&self.board).0 as i32;
        }

        let plys_to_zero = max_depth - ply;

        if !at_root {
            if alpha >= beta {
                return alpha
            }
        }

        if !is_pv
            && tt_hit
            && tt_entry.depth as u16 >= plys_to_zero
            && tt_value != 0
            && correct_bound_eq(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        if in_check {
            pos_eval = 0;
        } else {
            if tt_hit {
                if tt_entry.eval == 0 {
                    pos_eval = Eval::eval_low(&self.board).0 as i32;
                }
                if tt_value != 0 && correct_bound(tt_value, pos_eval, tt_entry.node_type()) {
                    pos_eval = tt_value;
                }
            } else {
                pos_eval = Eval::eval_low(&self.board).0 as i32;
                tt_entry.place(zob, BitMove::null(), 0, pos_eval as i16, 0, NodeBound::NoBound);
            }
        }

        if !in_check {

            if ply > 3
                && ply < 7
                && pos_eval - futility_margin(ply) >= beta && pos_eval < 10000 {
                return pos_eval;
            }
        }

        #[allow(unused_mut)]
        let mut moves: MoveList = if at_root {
            self.root_moves.to_list()
        } else {
            self.board.generate_pseudolegal_moves()
        };

        if moves.is_empty() {
            if self.board.in_check() {
                return Value::MATE.0 as i32 + (ply as i32);
            } else {
                return Value::DRAW.0 as i32;
            }
        }

        if !at_root {
            mvv_lva_sort(&mut moves, &self.board);
        }


        for (i, mov) in moves.iter().enumerate() {
            if at_root || self.board.legal_move(*mov) {
                moves_played += 1;
                let gives_check: bool = self.board.gives_check(*mov);
                self.board.apply_unknown_move(*mov, gives_check);
                let do_full_depth: bool = if max_depth >= 3 && moves_played > 1 && ply >= 2 {
                    if in_check || gives_check {
                        value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth - 1);
                    } else {
                        value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth - 2);
                    }
                    value > alpha
                } else {
                    !is_pv || moves_played > 1
                };
                if do_full_depth {
                    value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth);
                }
                if is_pv && (moves_played == 1 || (value > alpha && (at_root || value < beta))) {
                    value = -self.search::<PV>(-beta, -alpha, max_depth);
                }
                self.board.undo_move();
                assert!(value > Value::NEG_INFINITE.0 as i32);
                assert!(value < Value::INFINITE.0 as i32);
                if self.stop() {
                    return 0;
                }
                if at_root {
                    if moves_played == 1 || value as i32 > alpha {
                        self.root_moves.insert_score_depth(i,value, max_depth);
                    } else {
                        self.root_moves.insert_score(i, Value::NEG_INFINITE.0 as i32);
                    }
                }

                if value > best_value {
                    best_value = value;

                    if value > alpha {
                        best_move = *mov;
                        if is_pv && value < beta {
                            alpha = value;
                        } else {
//                            assert!(value >= beta);
                           break;
                        }
                    }
                }
            }
        }

        if moves_played == 0 {
            if self.board.in_check() {
                return Value::MATE.0 as i32 + (ply as i32);
            } else {
                return Value::DRAW.0 as i32;
            }
        }

        let node_bound = if best_value as i32 >= beta {NodeBound::LowerBound}
            else if is_pv && !best_move.is_null() {NodeBound::Exact}
                else {NodeBound::UpperBound};

        tt_entry.place(zob, best_move, best_value as i16, pos_eval as i16, plys_to_zero as u8, node_bound);

        best_value
    }

    // TODO: Qscience search

    fn main_thread(&self) -> bool {
        self.id == 0
    }

    fn stop(&self) -> bool {
        self.root_moves.load_stop()
    }

    fn check_time(&mut self) {
        if self.limit.use_time_management().is_some()
            && TIMER.elapsed() >= TIMER.maximum_time() {
            self.root_moves.set_stop(true);
        } else if let Some(time) = self.limit.use_movetime() {
            if self.limit.elapsed() >= time as i64 {
                self.root_moves.set_stop(true);
            }
        }
    }

    pub fn print_startup(&self) {
        if self.use_stdout() {
            println!("info id {} start", self.id);
        }
    }

    pub fn use_stdout(&self) -> bool {
        self.use_stdout.load(Ordering::Relaxed)
    }

}

fn mvv_lva_sort(moves: &mut MoveList, board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture() {
            piece.value() - board.captured_piece(*a).unwrap().value()
        } else if a.is_castle() {
            1
        } else if piece == Piece::P {
            if a.is_double_push().0 {
                2
            } else {
                3
            }
        } else {
            4
        }
    })
}

fn correct_bound_eq(tt_value: i32, beta: i32, bound: NodeBound) -> bool {
    if tt_value as i32 >= beta {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}

fn correct_bound(tt_value: i32, val: i32, bound: NodeBound) -> bool {
    if tt_value as i32 > val {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}


fn futility_margin(depth: u16) -> i32 {
    depth as i32 * 150
}