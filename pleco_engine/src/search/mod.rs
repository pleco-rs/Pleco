//! The main searching function.

pub mod eval;
pub mod movepick;

use std::cmp::{min,max};
use std::sync::atomic::{Ordering,AtomicBool};
use std::cell::UnsafeCell;
use std::mem;

use rand;
use rand::Rng;

use pleco::{MoveList,Board,BitMove};
use pleco::core::*;
use pleco::tools::tt::*;
use pleco::core::score::*;
use pleco::tools::pleco_arc::Arc;
//use pleco::board::movegen::{MoveGen,PseudoLegal};
//use pleco::core::mono_traits::{QuietChecksGenType};

use {MAX_PLY,TT_TABLE,THREAD_STACK_SIZE};

use threadpool::threadpool;
use time::time_management::TimeManager;
use time::uci_timer::*;
use threadpool::TIMER;
use sync::{GuardedBool,LockLatch};
use root_moves::RootMove;
use root_moves::root_moves_list::RootMoveList;
use tables::material::Material;
use tables::pawn_table::PawnTable;
use consts::*;


const THREAD_DIST: usize = 20;

//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [u16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [u16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

/// A Stack for the searcher, with information being contained per-ply.
pub struct ThreadStack {
    stack: [Stack; THREAD_STACK_SIZE],
}

impl ThreadStack {
    pub fn new() -> Self {
        unsafe {mem::zeroed()}
    }

    /// Get's a certain frame from the stack.
    ///
    /// Assumes the frame is within bounds, otherwise undefined behavior.
    pub fn get(&mut self, frame: usize) -> &mut Stack {
        debug_assert!(frame < THREAD_STACK_SIZE);
        unsafe {
            self.stack.get_unchecked_mut(frame)
        }
    }

    /// Get the ply at Zero
    pub fn ply_zero(&mut self) -> &mut Stack {
        self.get(4)
    }
}

pub struct Stack {
    pv: BitMove,
    ply: u16,
}

impl Stack {
    /// Get the next ply at an offset.
    pub fn offset(&mut self, count: isize) -> &mut Stack {
        unsafe {
            let ptr: *mut Stack = self as *mut Stack;
            &mut *ptr.offset(count)
        }
    }

    /// Get the next ply's Stack.
    pub fn incr(&mut self) -> &mut Stack {
        self.offset(1)
    }
}

pub struct Searcher {
    // Synchronization primitives
    pub id: usize,
    pub kill: AtomicBool,
    pub searching: Arc<GuardedBool>,
    pub cond: Arc<LockLatch>,

    // search data
    pub depth_completed: u16,
    pub limit: Limits,
    pub board: Board,
    pub time_man: &'static TimeManager,
    pub tt: &'static TranspositionTable,
    pub pawns: PawnTable,
    pub material: Material,
    pub root_moves: UnsafeCell<RootMoveList>,
    pub selected_depth: u16,
    pub last_best_move: BitMove,
    pub last_best_move_depth: u16,
    // MainThread Information
    pub previous_score: Value,
    pub best_move: BitMove,
    pub failed_low: bool,
    pub best_move_changes: f64,
    pub previous_time_reduction: f64,

}

unsafe impl Send for Searcher {}
unsafe impl Sync for Searcher {}

impl Searcher {
    /// Creates a new `Searcher` of an ID and condition to be released by.
    pub fn new(id: usize, cond: Arc<LockLatch>) -> Self {
        Searcher {
            id,
            kill: AtomicBool::new(false),
            searching: Arc::new(GuardedBool::new(true)),
            cond,
            depth_completed: 0,
            limit: Limits::blank(),
            board: Board::default(),
            time_man: &TIMER,
            tt: &TT_TABLE,
            pawns: PawnTable::new(16384),
            material: Material::new(8192),
            root_moves: UnsafeCell::new(RootMoveList::new()),
            selected_depth: 0,
            last_best_move: BitMove::null(),
            last_best_move_depth: 0,
            previous_score: 0,
            best_move: BitMove::null(),
            failed_low: false,
            best_move_changes: 0.0,
            previous_time_reduction: 0.0
        }
    }

    pub fn clear(&mut self) {
        self.pawns.clear();
        self.material.clear();
        self.previous_time_reduction = 0.0;
        self.previous_score = INFINITE;
    }

    /// Spins in idle loop, waiting for it's condition to unlock.
    pub fn idle_loop(&mut self) {
        self.searching.set(false);
        loop {
            self.cond.wait();
            if self.kill.load(Ordering::SeqCst) {
                return;
            }
            self.go();
        }
    }

    /// Starts the searchering. Assumes the Board and Limits are set
    fn go(&mut self) {
        self.searching.set(true);
        if self.main_thread() {
            // If we're main thread, wake up the other threads
            self.main_thread_go();
        } else {
            // otherwise, just search normally
            self.search_root();
        }
        // This is so the UCI interface knows the searcher is done.
        self.searching.set(false);
    }

    /// Main thread searching function.
    fn main_thread_go(&mut self) {
        // set the global limit
        if let Some(timer) = self.limit.use_time_management() {
            TIMER.init(self.limit.start.clone(), &timer, self.board.turn(), self.board.moves_played());
        }

        // Increment the TT search table.
        self.tt.new_search();
        // Start each of the threads!
        threadpool().thread_cond.set();

        // Search ourselves
        self.search_root();

        // Lock the other threads condition variable
        threadpool().thread_cond.lock();
        // Tell the threads to stop
        threadpool().set_stop(true);
        // Wait for all the non-main threads to finish searching.
        threadpool().wait_for_non_main();

        // iterate through each thread, and find the best move available (based on score)
        let mut best_move = self.root_moves().first().bit_move;
        let mut best_score = self.root_moves().first().score;
        if let LimitsType::Depth(_) = self.limit.limits_type  {
            let mut best_thread: &Searcher = &self;
            threadpool().threads.iter().map(|u| unsafe {&**u.get()}).for_each(|th| {
                let depth_diff = th.depth_completed as i32 - best_thread.depth_completed as i32;
                let score_diff = th.root_moves().first().score - best_thread.root_moves()[0].score;
                if score_diff > 0 && depth_diff >= 0 {
                    best_thread = th;
                }
            });
            best_move =  best_thread.root_moves().first().bit_move;
            best_score = best_thread.root_moves().first().score;
        }

        self.previous_score = best_score;
        self.best_move = best_move;

        // Cases where the MainTHread did not have the correct best move, display it.
        if self.use_stdout() && best_move != self.root_moves().first().bit_move {
            println!("info id 0 pv {}",self.root_moves().first().bit_move);
        }


        if self.use_stdout() {
            println!("bestmove {}", best_move.to_string());
        }

    }

    // The per thread searching function
    fn search_root(&mut self) {

        // Early return. This shouldn't notmally happen.
        if self.stop() {
            return;
        }

        // notify GUI that this thread is starting
        if self.use_stdout() {
            println!("info id {} start", self.id);
        }

        // If use a max_depth limit, use that as the max depth.
        let max_depth = if self.main_thread() {
            if let LimitsType::Depth(d) = self.limit.limits_type {
                d
            } else {
                MAX_PLY
            }
        } else {
            MAX_PLY
        };

        if self.main_thread() {
            if self.use_stdout() {
                println!("Time.. Max: {}, Ideal: {}", TIMER.maximum_time(), TIMER.ideal_time());
            }
            self.best_move_changes = 0.0;
            self.failed_low = false;
        }

        let start_ply: u16 = START_PLY[self.id % THREAD_DIST];
        let skip_size: u16 = SKIP_SIZE[self.id % THREAD_DIST];
        let mut depth: u16 = start_ply + 1;

        let mut delta: i32 = NEG_INFINITE as i32;
        #[allow(unused_assignments)]
        let mut best_value: i32 = NEG_INFINITE as i32;
        let mut alpha: i32 = NEG_INFINITE as i32;
        let mut beta: i32 = INFINITE as i32;

        let mut time_reduction: f64 = 1.0;
        let mut stack: ThreadStack = ThreadStack::new();
        let ss: &mut Stack = stack.ply_zero();

        // Shuffle (or possibly sort) the root moves so each thread searches different moves.
        self.shuffle();

        // Iterative deeping. Start at the base ply (determined by thread_id), and then increment
        // by the skip size after searching that depth. If searching for depth, non-main threads
        // will ignore the max_depth and instead wait for a stop signal.
        'iterative_deepening: while (!self.stop() || !self.main_thread()) && depth < max_depth {

            if self.main_thread() {
                self.best_move_changes *= 0.505;
                self.failed_low = false;
            }

            // rollback all the root moves, ala set the previous score to the current score.
            self.root_moves().rollback();

            let prev_best_score = self.root_moves()[0].prev_score;

            // Delta gives a bound in the iterative loop before re-searching that position.
            // Only applicable for a depth of 5 and beyond.
            if depth >= 5 {
                delta = 18;
                alpha = max(prev_best_score - delta, NEG_INFINITE);
                beta = min(prev_best_score + delta, INFINITE);
            }

            // Loop until we find a value that is within the bounds of alpha, beta, and the delta margin.
            'aspiration_window: loop {
                // search!
                best_value = self.search::<PV>(alpha, beta, ss,depth);

                // Order root moves by the socre retreived post search.
                self.root_moves().sort();

                if self.stop() {
                    break 'aspiration_window;
                }

                // Check for incorrect search window. If the value if less than alpha
                // or greater than beta, we need to increase the search window and re-search.
                // Otherwise, go to the next search
                if best_value <= alpha {
                    beta = (alpha + beta) / 2;
                    alpha = max(best_value - delta, NEG_INFINITE);
                    if self.main_thread() {
                        self.failed_low = true;
                    }
                } else if best_value >= beta {
                    beta = min(best_value + delta, INFINITE);
                } else {
                    break 'aspiration_window;
                }
                delta += (delta / 4) + 5;

                assert!(alpha >= NEG_INFINITE);
                assert!(beta <= INFINITE);
            }

            // Main Thread provides an update to the GUI
            if self.use_stdout() && self.main_thread() {
                println!("info depth {} score {} pv {}",
                         depth,
                         best_value,
                         self.root_moves().first().bit_move.to_string());
            }


            if !self.stop() {
                self.depth_completed = depth;
            }

            let curr_best_move = unsafe {
                (*self.root_moves.get()).first().bit_move
            };

            if curr_best_move != self.last_best_move {
                    self.last_best_move = curr_best_move;
                    self.last_best_move_depth = depth;
            }

            depth += skip_size;

            if !self.main_thread() {
                continue;
            }

            // Main thread only from here on!


            // check for time
            if let Some(_) = self.limit.use_time_management() {
                if !self.stop() {
                    let score_diff: i32 = best_value - self.previous_score;

                    let improving_factor: i64 = (229).max((715).min(
                          357
                        + 119 * self.failed_low as i64
                        -   6 * score_diff as i64));

                    time_reduction = 1.0;

                    // If the bestMove is stable over several iterations, reduce time accordingly
                    for i in 3..6 {
                        if self.last_best_move_depth * i < self.depth_completed {
                            time_reduction *= 1.34;
                        }
                    }

                    // Use part of the gained time from a previous stable move for the current move
                    let mut unstable_factor: f64 = 1.0 + self.best_move_changes;
                    unstable_factor *= self.previous_time_reduction.powf(0.51) / time_reduction;

                    // Stop the search if we have only one legal move, or if available time elapsed
                    if self.root_moves().len() == 1
                        || TIMER.elapsed() >= (TIMER.ideal_time() as f64 * unstable_factor as f64 * improving_factor as f64 / 602.0) as i64 {
                        threadpool().set_stop(true);
                    }
                }
            }
        }

        if self.main_thread() {
            self.previous_time_reduction = time_reduction;
        }
    }

    // The searching function for a specific depth.
    fn search<N: PVNode>(&mut self, mut alpha: i32, beta: i32, ss: &mut Stack, depth: u16) -> i32 {
//        assert!(depth >= 1);
        let is_pv: bool = N::is_pv();
        let ply: u16 = ss.ply;
        let at_root: bool = ply == 0;
        let zob: u64 = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value: Value = if tt_hit {tt_entry.score as i32} else {0};
        let in_check: bool = self.board.in_check();

        let mut best_move = BitMove::null();

        let mut value: Value = NEG_INFINITE;
        let mut best_value: Value = NEG_INFINITE;
        let mut moves_played = 0;

        let mut pos_eval: i32 = 0;

        if self.main_thread() {
            self.check_time();
        }

        if depth == 0 || ply == MAX_PLY {
            if in_check {
                if ply != MAX_PLY {
                    return self.qsearch::<NonPV,InCheck>(alpha, alpha+1, ss, 0);
                } else {
                    return ZERO;
                }
            } else {
                return self.eval();
            }
        }


        // increment the next ply
        ss.incr().ply = ply + 1;

        if !at_root {
            if alpha >= beta {
                return alpha
            }
        }


        if !is_pv
            && tt_hit
            && tt_entry.depth as u16 >= depth
            && tt_value != 0
            && correct_bound_eq(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        // Get and set the position eval
        if in_check {
            pos_eval = 0;
        } else {
            if tt_hit {
                if tt_entry.eval == 0 {
                    pos_eval = self.eval();
                }
                if tt_value != 0 && correct_bound(tt_value, pos_eval, tt_entry.node_type()) {
                    pos_eval = tt_value;
                }
            } else {
                pos_eval = self.eval();
                tt_entry.place(zob, BitMove::null(),
                               0, pos_eval as i16,
                               0, NodeBound::NoBound,
                               self.tt.time_age());
            }

            // Futility Pruning
            if !at_root
                && depth < 7
                && pos_eval - futility_margin(depth) >= beta
                && pos_eval < 10000 {
                return pos_eval;
            }
        }


        #[allow(unused_mut)]
        let mut moves: MoveList = if at_root {
            self.root_moves().iter().map(|r| r.bit_move).collect()
        } else {
            self.board.generate_pseudolegal_moves()
        };

        if moves.is_empty() {
            if self.board.in_check() {
                return -MATE as i32 + (ply as i32);
            } else {
                return DRAW as i32;
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
                self.tt.prefetch(self.board.zobrist());
                let do_full_depth: bool = if depth >= 4 && moves_played > 1 && !mov.is_capture() && !mov.is_promo() {
                    let new_depth = if in_check || gives_check {depth - 2} else {depth - 3};
                    value = -self.search::<NonPV>(-(alpha+1), -alpha, ss.incr(), new_depth);
                    value > alpha
                } else {
                    !is_pv || moves_played > 1
                };
                if do_full_depth {
                    value = if depth <= 1 {
                        if gives_check { -self.qsearch::<NonPV,InCheck>(-(alpha+1), -alpha, ss.incr(), 0)
                        } else { -self.qsearch::<NonPV,NoCheck>(-(alpha+1), -alpha, ss.incr(), 0) }
                    } else {
                        -self.search::<NonPV>(-(alpha+1), -alpha, ss.incr(),  depth - 1)
                    };
                }

                if is_pv && (moves_played == 1 || (value > alpha && (at_root || value < beta))) {
                    value = if depth <= 1 {
                        if gives_check { -self.qsearch::<PV,InCheck>(-(alpha+1), -alpha, ss.incr(), 0)}
                            else {    -self.qsearch::<PV,NoCheck>(-(alpha+1), -alpha, ss.incr(), 0)}
                    } else {
                        -self.search::<PV>(-beta, -alpha, ss.incr(),depth -1)
                    };
                }

                self.board.undo_move();
                assert!(value > NEG_INFINITE);
                assert!(value < INFINITE );

                if self.stop() {
                    return 0;
                }

                if at_root {
                    let mut incr_bmc: bool = false;
                    {
                        let rm: &mut RootMove = unsafe { self.root_moves().get_unchecked_mut(i) };
                        if moves_played == 1 || value > alpha {
                            rm.depth_reached = depth;
                            rm.score = value;
                            if moves_played > 1 && self.main_thread() {
                                incr_bmc = true;
                            }
                        } else {
                            rm.score = NEG_INFINITE;
                        }
                    }
                    if incr_bmc {
                        self.best_move_changes += 1.0;
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

        if moves_played == 0 {
            if self.board.in_check() {
                return MATE as i32 - (ply as i32);
            } else {
                return DRAW as i32;
            }
        }

        let node_bound = if best_value as i32 >= beta {NodeBound::LowerBound}
            else if is_pv && !best_move.is_null() {NodeBound::Exact}
                else {NodeBound::UpperBound};


        tt_entry.place(zob, best_move, best_value as i16,
                       pos_eval as i16, depth as i16,
                       node_bound, self.tt.time_age());

        best_value
    }

    // TODO: Qscience search
    fn qsearch<N: PVNode, C: CheckState>(&mut self, mut alpha: i32, beta: i32, ss: &mut Stack, rev_depth: i16) -> i32 {
        let is_pv: bool = N::is_pv();
        let in_check: bool = C::in_check();

        assert!(alpha >= NEG_INFINITE);
        assert!(beta <= INFINITE);
        assert!(alpha < beta);
        assert!(rev_depth <= 0);
        assert!(is_pv || (alpha == beta - 1));
        assert_eq!(in_check, self.board.in_check());

        if in_check != self.board.in_check() {
            self.board.pretty_print();
        }


        let ply: u16 = ss.ply;
        let zob: u64 = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value: Value = if tt_hit {tt_entry.score as i32} else {0};

        let mut best_move = BitMove::null();

        let mut value: Value;
        let mut best_value: Value = NEG_INFINITE;
        let mut pos_eval: Value = NEG_INFINITE + 1;
        let mut moves_played = 0;
        let old_alpha = alpha;
        let tt_depth: i16 = if in_check || rev_depth >= 0 {0} else {-1};


        if ply >= MAX_PLY {
            if !in_check {
                return self.eval();
            } else {
                return ZERO;
            }
        }

        // increment the next ply
        ss.incr().ply = ply + 1;

        if !is_pv
            && tt_hit
            && tt_entry.depth as i16 >= tt_depth
            && tt_value != 0
            && correct_bound_eq(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        if !in_check {
            if tt_hit {
                if tt_entry.eval == 0 {
                    best_value = self.eval();
                    pos_eval = best_value;
                } else {
                    best_value = tt_entry.eval as i32;
                    pos_eval = best_value;
                }

                if tt_value != 0 && correct_bound(tt_value, best_value, tt_entry.node_type()) {
                    best_value == tt_value;
                }
            } else {
                best_value = self.eval();
                pos_eval = best_value;
            }

            if best_value >= beta {
                if !tt_hit {
                    tt_entry.place(zob, BitMove::null(), best_value as i16,
                                   pos_eval as i16, -6,
                                   NodeBound::LowerBound, self.tt.time_age());
                }
                return best_value;
            }

            if is_pv && best_value > alpha {
                alpha = best_value;
            }
        }

        let moves: MoveList = if in_check {
            self.board.generate_pseudolegal_moves_of_type(GenTypes::Evasions)
        } else {
            self.board.generate_pseudolegal_moves_of_type(GenTypes::Captures)
        };

        for  mov in moves.iter() {
            if self.board.legal_move(*mov) {
                moves_played += 1;
                let gives_check: bool = self.board.gives_check(*mov);
                self.board.apply_unknown_move(*mov, gives_check);
                self.tt.prefetch(self.board.zobrist());
                assert_eq!(gives_check, self.board.in_check());

                value = if gives_check {
                    -self.qsearch::<N,InCheck>(-beta, -alpha, ss.incr(),rev_depth - 1)
                } else {
                    -self.qsearch::<N,NoCheck>(-beta, -alpha, ss.incr(),rev_depth - 1)
                };

                self.board.undo_move();

                if value <= NEG_INFINITE {
                    println!("Value {}, move {}", value, *mov);
                    self.board.pretty_print();

                }
                assert!(value > NEG_INFINITE);
                assert!(value < INFINITE );

                if value > best_value {
                    best_value = value;

                    if value > alpha {

                        if is_pv && value < beta {
                            best_move = *mov;
                            alpha = value;
                        } else {
                            tt_entry.place(zob, best_move, best_value as i16,
                                           pos_eval as i16, tt_depth as i16,
                                           NodeBound::LowerBound, self.tt.time_age());
                            return value;
                        }
                    }
                }
            }
        }

        if moves_played == 0 {
            if self.board.in_check() {
                return -MATE as i32 + (ply as i32);
            } else {
                return pos_eval as i32;
            }
        }

        let node_bound = if  is_pv && best_value > old_alpha {NodeBound::Exact}
                else {NodeBound::UpperBound};


        tt_entry.place(zob, best_move, best_value as i16,
                       pos_eval as i16, tt_depth,
                       node_bound, self.tt.time_age());


        assert!(best_value > NEG_INFINITE);
        assert!(best_value < INFINITE );
        best_value
    }


    pub fn eval(&mut self) -> Value {
        let pawns = &mut self.pawns;
        let material = &mut self.material;
        eval::Evaluation::evaluate(&self.board, pawns, material)
    }

    #[inline(always)]
    fn main_thread(&self) -> bool {
        self.id == 0
    }

    #[inline(always)]
    fn stop(&self) -> bool {
        threadpool().stop.load(Ordering::Relaxed)
    }

    fn check_time(&mut self) {
        if self.limit.use_time_management().is_some()
            && TIMER.elapsed() >= TIMER.maximum_time() {
            threadpool().set_stop(true);
        } else if let Some(time) = self.limit.use_movetime() {
            if self.limit.elapsed() >= time as i64 {
                threadpool().set_stop(true);
            }
        }
    }

    #[inline(always)]
    pub fn print_startup(&self) {
        if self.use_stdout() {
            println!("info id {} start", self.id);
        }
    }

    #[inline(always)]
    pub fn use_stdout(&self) -> bool {
        USE_STDOUT.load(Ordering::Relaxed)
    }

    pub fn shuffle(&mut self) {
        if self.id == 0 || self.id >= 20 {
            self.rm_mvv_laa_sort();
        } else {
            rand::thread_rng().shuffle(self.root_moves().as_mut());
        }
    }

    #[inline]
    pub fn root_moves(&self) -> &mut RootMoveList {
        unsafe {
            &mut *self.root_moves.get()
        }
    }

    #[inline]
    fn rm_mvv_laa_sort(&mut self) {
        let board = &self.board;
        self.root_moves().sort_by_key(|root_move| {
            let a = root_move.bit_move;
            let piece = board.piece_at_sq((a).get_src()).unwrap();

            if a.is_capture() {
                piece.value() - board.captured_piece(a).unwrap().value()
            } else if a.is_castle() {
                1
            } else if piece == PieceType::P {
                if a.is_double_push().0 {
                    2
                } else {
                    3
                }
            } else {
                4
            }
        });
    }
}


impl Drop for Searcher {
    fn drop(&mut self) {
        self.searching.set(false);
    }
}

fn mvv_lva_sort(moves: &mut MoveList, board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture() {
            piece.value() - board.captured_piece(*a).unwrap().value()
        } else if a.is_castle() {
            1
        } else if piece == PieceType::P {
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

#[inline]
fn futility_margin(depth: u16) -> i32 {
    depth as i32 * 150
}