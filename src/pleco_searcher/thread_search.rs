use Board;
use super::threads::Thread;
use super::UCILimit;

use rand::{Rng,self};

//use test::{self,Bencher};
use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};

use std::cmp::{min,max};

use board::*;
use core::*;
use board::eval::*;
use core::piece_move::BitMove;
use tools::tt::*;
use engine::*;

use super::misc::*;
use super::{LIMIT,TT_TABLE,THREAD_STACK_SIZE,MAX_PLY};

const THREAD_DIST: usize = 20;
//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [u16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [u16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];


pub struct ThreadSearcher<'a> {
    pub thread: &'a mut Thread,
    pub limit: UCILimit,
    pub board: Board
}

impl<'a> ThreadSearcher<'a> {
    pub fn search_root(&mut self) {
        println!("info id {} start", self.thread.id);

        let max_depth = if self.limit.is_depth() {
            self.limit.depth_limit()
        } else {
            MAX_PLY
        };

        let start_ply: u16 = START_PLY[self.thread.id % THREAD_DIST];
        let skip_size: u16 = SKIP_SIZE[self.thread.id % THREAD_DIST];
        let mut depth: u16 = start_ply;
        let mut delta = 31;

        #[allow(unused_assignments)]
        let mut best_value: i16 = NEG_INFINITY;
        let mut alpha = NEG_INFINITY;
        let mut beta = INFINITY;

        self.shuffle_root_moves();

        while !self.stop() && depth <= max_depth {
            if depth != start_ply {
                self.sort_root_moves();
            }

            if depth >= 5 {
                delta = 18;
                alpha = best_value - delta;
                beta = best_value + delta;
            }

            'aspiration_window: loop {
                best_value = self.search::<PV>(alpha, beta, depth);
                self.sort_root_moves();

                if self.stop() {
                    break 'aspiration_window;
                }

                if best_value <= alpha {
                    beta = (alpha + beta) / 2;
                    if (best_value.wrapping_sub(delta)) > best_value {
                        alpha = NEG_INFINITY;
                    } else {
                        alpha = max(best_value - delta, NEG_INFINITY);
                    }
                } else if best_value >= beta {
                    if (best_value.wrapping_add(delta)) < best_value {
                        beta = INFINITY;
                    } else {
                        beta = min(best_value + delta, INFINITY);
                    }
                } else {
                    break 'aspiration_window;
                }

                if delta.wrapping_add((delta / 4) + 5) > 0 {
                    delta += (delta / 4) + 5;
                }
            }

            self.sort_root_moves();
            if self.thread.use_stdout.load(Ordering::Relaxed) {
                println!("info id {} depth {} stop {}",self.thread.id, depth, self.stop());
            }
            depth += skip_size;
        }
    }

    fn search<N: PVNode>(&mut self, mut alpha: i16, beta: i16, max_depth: u16) -> i16 {

        let is_pv: bool = N::is_pv();
        let old_alpha = alpha;
        let at_root: bool = self.board.depth() == 0;
        let zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value = if tt_hit {tt_entry.score} else {0};
        let in_check: bool = self.board.in_check();
        let ply = self.board.depth();

        let mut pos_eval = 0;

        if self.board.depth() == max_depth {
            return Eval::eval_low(&self.board);
        }

        if !is_pv
            && tt_hit
            && tt_entry.depth >= max_depth as u8 // TODO: Fix this hack
            && tt_entry.best_move != BitMove::null()
            && tt_value != 0
            && correct_bound(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        if in_check {
            pos_eval = 0;
            self.thread.thread_stack[ply as usize].pos_eval = pos_eval;
        } else if tt_hit {
            // update Evaluation
            if tt_entry.eval == 0 {
                pos_eval = Eval::eval_low(&self.board);
                self.thread.thread_stack[ply as usize].pos_eval = pos_eval;
            } else {
                pos_eval = tt_entry.eval;
                self.thread.thread_stack[ply as usize].pos_eval = pos_eval;
            }
        } else {
            pos_eval = Eval::eval_low(&self.board);
            tt_entry.place(zob, BitMove::null(), 0, pos_eval, 0, NodeBound::NoBound);
        }

        if !in_check {

            // futility pruning
            if !at_root && ply < 7 && pos_eval - (150 * ply as i16) >= beta && pos_eval < 10000 {
                return pos_eval;
            }
        }

        #[allow(unused_mut)]
        let mut moves: Vec<BitMove> = if at_root {
            self.thread.root_moves.read().unwrap().iter().map(|m| m.bit_move).collect()
        } else {
            self.board.generate_pseudolegal_moves()
        };

        if moves.is_empty() {
            if self.board.in_check() {
                return MATE + (self.board.depth() as i16);
            } else {
                return -STALEMATE;
            }
        }

        let mut best_move = BitMove::null();

        let mut value = 0;
        let mut best_value = NEG_INFINITY;

        for (i, mov) in moves.iter().enumerate() {
            if at_root || self.board.legal_move(*mov) {
                self.board.apply_move(*mov);
                if is_pv && (i == 0 || (value > alpha && (at_root || value < beta))) {
                    value = -self.search::<PV>(-beta, -alpha,max_depth);
                } else {
                    value = -self.search::<NonPV>(-beta, -alpha,max_depth);
                }
                self.board.undo_move();
                if self.stop() {
                    return 0;
                }
                if at_root {
                    let mut moves = self.thread.root_moves.write().unwrap();
                    let rootmove: &mut RootMove = moves.get_mut(i).unwrap();
                    if (i == 0 || value > alpha) {
                        rootmove.rollback_insert(value, max_depth);
                    } else {
                        rootmove.score = NEG_INFINITY;
                    }
                }

                if value > best_value {
                    best_value = value;

                    if value > alpha {
                        best_move = *mov;
                        if is_pv && value < beta {
                            alpha = value;
                        } else {
                           break;
                        }
                    }
                }
            }
        }

        let node_bound = if best_value >= beta {NodeBound::LowerBound}
            else if is_pv && !best_move.is_null() {NodeBound::Exact}
                else {NodeBound::UpperBound};
        tt_entry.place(zob,best_move,best_value, pos_eval,max_depth as u8 - ply as u8, node_bound);

        best_value
    }

    fn main_thread(&self) -> bool {
        self.thread.id == 0
    }

    fn stop(&self) -> bool {
        self.thread.stop.load(Ordering::Relaxed)
    }

    pub fn print_startup(&self) {
        if self.thread.use_stdout.load(Ordering::Relaxed) {
            println!("info id {} start", self.thread.id);
        }
    }

    fn sort_root_moves(&mut self) {
        let mut moves = self.thread.root_moves.write().unwrap();
        (*moves).sort();
    }

    fn shuffle_root_moves(&mut self) {
        if self.main_thread() || self.thread.id >= 20 {
            self.thread.root_moves.write().unwrap().sort_by_key(|root_move| {
                let a = root_move.bit_move;
                let piece = self.board.piece_at_sq((a).get_src()).unwrap();

                if a.is_capture() {
                    self.board.captured_piece(a).unwrap().value() - piece.value()
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
        } else {
            let mut moves = self.thread.root_moves.write().unwrap();
            let slice = moves.as_mut_slice();
            rand::thread_rng().shuffle(slice);
        }
    }
}


fn correct_bound(tt_value: i16, beta: i16, bound: NodeBound) -> bool {
    if tt_value >= beta {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}