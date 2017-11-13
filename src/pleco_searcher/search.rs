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

pub struct Thread {
    pub board: Board,
    pub root_moves: Arc<RwLock<Vec<RootMove>>>,
    pub id: usize,
    pub tt: &'static TT,
    pub nodes: Arc<AtomicU64>,
    pub local_stop: Arc<AtomicBool>,
    pub cond_var: Arc<(Mutex<bool>,Condvar)>,
    pub thread_stack: [ThreadStack; THREAD_STACK_SIZE],
    pub limit: UCILimit,
}

impl Thread {
    pub fn new(board: &Board, moves: Arc<RwLock<Vec<RootMove>>>, id: usize,
    nodes: &Arc<AtomicU64>, stop: &Arc<AtomicBool>, cond_var: &Arc<(Mutex<bool>,Condvar)>)
        -> Thread {
        Thread {
            board: board.parallel_clone(),
            root_moves: moves,
            id: id,
            tt: &TT_TABLE,
            nodes: Arc::clone(nodes),
            local_stop: Arc::clone(stop),
            cond_var: Arc::clone(cond_var),
            thread_stack: init_thread_stack(),
            limit: UCILimit::Infinite,
        }
    }


    pub fn idle_loop(&mut self) {
        {
            let &(ref lock, ref cvar) = &*(Arc::clone(&self.cond_var));
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
        }
        self.limit = unsafe {LIMIT.clone()};
        self.thread_search();
    }

    fn stop(&self) -> bool {
        self.local_stop.load(Ordering::Relaxed)
    }

    pub fn thread_search(&mut self) {
        self.shuffle_root_moves();

        println!("info id {} start",self.id);

        let max_depth = if self.limit.is_depth() {
            self.limit.depth_limit()
        } else {
            MAX_PLY
        };

        let start_ply: u16 = START_PLY[self.id % THREAD_DIST];
        let skip_size: u16 = SKIP_SIZE[self.id % THREAD_DIST];
        let mut depth: u16 = start_ply;
        let mut delta = 31;

        #[allow(unused_assignments)]
        let mut best_value: i16 = NEG_INFINITY;
        let mut alpha = NEG_INFINITY;
        let mut beta = INFINITY;

        while !self.stop() && depth <= max_depth {
            if depth != start_ply {
                self.sort_root_moves();
            }

            'aspiration_window: loop {
                best_value = self.search::<PV>(alpha, beta, depth);
                self.sort_root_moves();

                if self.stop() {
                    break 'aspiration_window;
                }

                if best_value <= alpha {
                    beta = (alpha + beta) / 2;
                    alpha = max(best_value - delta, NEG_INFINITY);
                } else if best_value >= beta {
                    beta = min(best_value + delta, INFINITY);
                } else {
                    break 'aspiration_window;
                }

                delta += (delta / 4) + 5;
            }

            self.sort_root_moves();

            //            if self.limit.use_time() {
            //                // DO SOMETHING
            //            }

            println!("info id {} depth {} stop {}",self.id, depth, self.stop());

            depth += skip_size;
        }
    }

    fn search<N: PVNode>(&mut self, mut alpha: i16, beta: i16, max_depth: u16) -> i16 {

        let is_pv: bool = N::is_pv();
        let at_root: bool = self.board.ply() == 0;
        let zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value = if tt_hit {tt_entry.score} else {0};
        let in_check: bool = self.board.in_check();
        let ply = self.board.ply();

        let mut pos_eval = 0;

        if self.board.depth() == max_depth {
            return Eval::eval_low(&self.board);
        }

        if !is_pv
            && tt_hit
            && tt_entry.depth >= self.board.depth() as u8 // TODO: Fix this hack
            && tt_value != 0
            && correct_bound(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        if in_check {
            pos_eval = 0;
            self.thread_stack[ply as usize].pos_eval = pos_eval;
        } else if tt_hit {

            // update Evaluation
            if tt_entry.eval == 0 {
                pos_eval = Eval::eval_low(&self.board);
                self.thread_stack[ply as usize].pos_eval = pos_eval;
            } else {
                pos_eval = tt_entry.eval;
                self.thread_stack[ply as usize].pos_eval = pos_eval;
            }

        }

        #[allow(unused_mut)]
        let mut moves: Vec<BitMove> = if at_root {
            self.root_moves.read().unwrap().iter().map(|m| m.bit_move).collect()
        } else {
            self.board.generate_moves()
        };

        if moves.is_empty() {
            if self.board.in_check() {
                return MATE + (self.board.depth() as i16);
            } else {
                return -STALEMATE;
            }
        }

        for (i, mov) in moves.iter().enumerate() {

            self.board.apply_move(*mov);
            let ret_mov = -self.search::<PV>(-beta, -alpha,max_depth);
            self.board.undo_move();
            if at_root {
                let mut moves = self.root_moves.write().unwrap();
                moves.get_mut(i).unwrap().rollback_insert(ret_mov,max_depth);
            }

            if ret_mov > alpha {
                alpha = ret_mov;
            }

            if alpha > beta {
                return alpha;
            }

            if self.stop() {
                return 0;
            }
        }
        alpha
    }

    fn qsearch(&mut self, _alpha: i16, _beta: i16, _max_depth: u16) -> i16 {
        unimplemented!()
    }

    fn main_thread(&self) -> bool {
        self.id == 0
    }

    fn sort_root_moves(&mut self) {
        let mut moves = self.root_moves.write().unwrap();
        moves.sort();
    }

    fn shuffle_root_moves(&mut self) {
        if self.main_thread() || self.id >= 20 {
            self.root_moves.write().unwrap().sort_by_key(|root_move| {
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
            let mut moves = self.root_moves.write().unwrap();
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