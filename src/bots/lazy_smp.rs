use num_cpus;
use rand::{Rng,self};

//use test::{self,Bencher};
use std::sync::{Arc,Mutex,Condvar,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::thread::{JoinHandle,self};
use std::cmp::{min,max,Ordering as CmpOrder};
use std::mem;

use board::*;
use templates::*;
use eval::*;
use piece_move::BitMove;
use tt::*;
use engine::*;

const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
const THREAD_DIST: usize = 20;

//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [u16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [u16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

static mut LIMIT: UCILimit = UCILimit::Infinite;

lazy_static! {
    pub static ref TT_TABLE: TT = TT::new(1000);
}

trait PVNode {
    fn is_pv() -> bool;
}

struct PV {}
struct NonPV {}

impl PVNode for PV {
    fn is_pv() -> bool {
        true
    }
}

impl PVNode for NonPV {
    fn is_pv() -> bool {
        false
    }
}


#[derive(Copy, Clone, Eq)]
struct RootMove {
    pub bit_move: BitMove,
    pub score: i16,
    pub prev_score: i16,
    pub depth_reached: u16
}

// Moves with higher score for a higher depth are less
impl Ord for RootMove {
    fn cmp(&self, other: &RootMove) -> CmpOrder {
        let value_diff = self.score as i16 - other.score as i16;
        if value_diff > 0 {
            let depth_diff = self.depth_reached as i16 - other.depth_reached as i16;
            if depth_diff == 0 {
                return CmpOrder::Equal;
            } else if depth_diff > 0 {
                return CmpOrder::Less;
            }
        }
        CmpOrder::Greater
    }
}

impl PartialOrd for RootMove {
    fn partial_cmp(&self, other: &RootMove) -> Option<CmpOrder> {
        Some(self.cmp(other))
    }
}


impl PartialEq for RootMove {
    fn eq(&self, other: &RootMove) -> bool {
        self.score == other.score && self.depth_reached == other.depth_reached
    }
}



impl RootMove {
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move: bit_move,
            score: NEG_INFINITY,
            prev_score: NEG_INFINITY,
            depth_reached: 0
        }
    }

    pub fn rollback_insert(&mut self, score: i16, depth: u16) {
        self.prev_score = self.score;
        self.score = score;
        self.depth_reached = depth;
    }
}


struct ThreadStack {
    pub pos_eval: i16,
}

impl ThreadStack {
    pub fn new() -> Self {
        ThreadStack {
            pos_eval: 0
        }
    }
}

struct Thread {
    board: Board,
    root_moves: Arc<RwLock<Vec<RootMove>>>,
    id: usize,
    tt: &'static TT,
    nodes: Arc<AtomicU64>,
    local_stop: Arc<AtomicBool>,
    cond_var: Arc<(Mutex<bool>,Condvar)>,
    thread_stack: [ThreadStack; THREAD_STACK_SIZE],
    limit: UCILimit,
}

impl Thread {
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
                    value_of_piece(self.board.captured_piece(a).unwrap()) - value_of_piece(piece)
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


pub struct LazySMPSearcher {
    board: Board,
    gui_stop: Arc<AtomicBool>,
    cond_var: Arc<(Mutex<bool>,Condvar)>,
    all_moves: Vec<Arc<RwLock<Vec<RootMove>>>>,
    threads: Vec<JoinHandle<()>>,
    main_thread: Thread,
}

const DEFAULT_TT_CAP: usize = 100000000;

impl LazySMPSearcher {
    pub fn setup(board: Board, stop: Arc<AtomicBool>) -> Self {
        let num_threads = max(num_cpus::get(),1);

        let nodes = Arc::new(AtomicU64::new(0));
        let cond_var = Arc::new((Mutex::new(false), Condvar::new()));

        let root_moves: Vec<RootMove> = board.generate_moves().into_iter().map(|m| RootMove::new(m)).collect();


        let mut all_moves = Vec::with_capacity(num_threads);

        let mut threads = Vec::with_capacity(num_threads);

        let main_thread_moves = Arc::new(RwLock::new(root_moves.clone()));
        all_moves.push(main_thread_moves.clone()); // index 0, aka the main thread

        for x in 1..num_threads {
            let builder = thread::Builder::new();
            let shared_moves = Arc::new(RwLock::new(root_moves.clone()));
            all_moves.push(shared_moves.clone());

            let new_thread = Thread {
                board: board.parallel_clone(),
                root_moves: shared_moves,
                id: x,
                tt: &TT_TABLE,
                nodes: Arc::clone(&nodes),
                local_stop: Arc::clone(&stop),
                cond_var: Arc::clone(&cond_var),
                thread_stack: init_thread_stack(),
                limit: UCILimit::Infinite,
            };

            let join_handle = builder.spawn(move || {
                let mut current_thread = new_thread;
                current_thread.idle_loop()
            }).unwrap();

            threads.push(join_handle);

        }

        let main_thread = Thread {
            board: board.parallel_clone(),
            root_moves: main_thread_moves,
            id: 0,
            tt: &TT_TABLE,
            nodes: Arc::clone(&nodes),
            local_stop: Arc::clone(&stop),
            cond_var: Arc::clone(&cond_var),
            thread_stack: init_thread_stack(),
            limit: UCILimit::Infinite,
        };

        let lazy_smp = LazySMPSearcher {
            board: board,
            gui_stop: stop,
            cond_var: cond_var,
            all_moves: all_moves,
            threads: threads,
            main_thread: main_thread,
        };
        lazy_smp
    }

    pub fn start_searching(&mut self, limit: UCILimit, use_stdout: bool) -> BitMove {
        // Make sure there is no stop command
        assert!(!(self.gui_stop.load(Ordering::Relaxed)));

        // Check if Moves is empty
        {
            if self.main_thread.root_moves.read().unwrap().is_empty() {
                return BitMove::null();
            }
        }

        // Set the global limit
        unsafe {
            LIMIT = limit;
        }

        // get cond_var and notify the threads to wake up
        {
            let &(ref lock, ref cvar) = &*(self.cond_var.clone());
            let mut started = lock.lock().unwrap();
            *started = true;
            cvar.notify_all();

        }

        // Main thread needs to start searching
        self.main_thread.thread_search();


        // Make sure the remaining threads have finished.
        while !self.threads.is_empty() {
            self.threads.pop().unwrap().join().unwrap();
        }

        let mut best_root_move: RootMove = {
            self.main_thread.root_moves.read().unwrap().get(0).unwrap().clone()
        };



        // Find out if there is a better found move
        for thread_moves in self.all_moves.iter() {
            let root_move: RootMove = thread_moves.read().unwrap().get(0).unwrap().clone();
            let depth_diff = root_move.depth_reached as i16 - best_root_move.depth_reached as i16;
            let value_diff = root_move.score as i16 - best_root_move.score as i16;

            // If it has a bigger value and greater or equal depth
            if value_diff > 0 && depth_diff >= 0 {
                best_root_move = root_move;
            }
        }

        if use_stdout {
            println!("bestmove {}", best_root_move.bit_move);
        }

        best_root_move.bit_move
    }

    pub fn perft(depth: u16) -> u64 {
        unimplemented!()
    }
}

impl Drop for LazySMPSearcher {
    fn drop(&mut self) {
        while !self.threads.is_empty() {
            let thread_handle = self.threads.pop().unwrap();
            thread_handle.join().unwrap();
        }
    }
}




impl Searcher for LazySMPSearcher {
    fn name() -> &'static str {
        "Lazy SMP Searcher"
    }

    fn best_move(board: Board, limit: UCILimit) -> BitMove {
        let mut searcher = LazySMPSearcher::setup(board,Arc::new(AtomicBool::new(false)));
        searcher.start_searching(limit, false)
    }
}

fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}

impl UCISearcher for LazySMPSearcher {
    fn uci_setup(board: Board, stop: Arc<AtomicBool>) -> Self {
        LazySMPSearcher::setup(board,stop)
    }

    fn uci_go(&mut self, limits: UCILimit, _use_stdout: bool) -> BitMove {
        self.start_searching(limits, true)
    }
}