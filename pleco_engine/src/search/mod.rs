//! The main searching function.

pub mod eval;

use std::cell::UnsafeCell;
use std::cmp::{max, min};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use pleco::core::piece_move::MoveType;
use pleco::core::score::*;
use pleco::core::*;
use pleco::helper::prelude::*;
use pleco::tools::pleco_arc::Arc;
use pleco::tools::tt::*;
use pleco::tools::PreFetchable;
use pleco::{BitMove, Board, SQ};
//use pleco::board::movegen::{MoveGen,PseudoLegal};
//use pleco::core::mono_traits::{QuietChecksGenType};

use {MAX_PLY, THREAD_STACK_SIZE};

use consts::*;
use movepick::MovePicker;
use root_moves::root_moves_list::RootMoveList;
use root_moves::RootMove;
use sync::{GuardedBool, LockLatch};
use tables::material::Material;
use tables::pawn_table::PawnTable;
use tables::prelude::*;
use threadpool::threadpool;
use time::time_management::TimeManager;
use time::uci_timer::*;

const RAZORING_MARGIN: i32 = 590;

const THREAD_DIST: usize = 20;

//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [i16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [i16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

static mut REDUCTIONS: [[[[i16; 64]; 64]; 2]; 2] = [[[[0; 64]; 64]; 2]; 2]; // [pv][improving][depth][moveNumber]
static mut FUTILITY_MOVE_COUNTS: [[i32; 16]; 2] = [[0; 16]; 2]; // [improving][depth]
static RAZOR_MARGIN: [i32; 3] = [0, 590, 604];

static CAPTURE_PRUNE_MARGIN: [i32; 7] = [
    0,
    1 * PAWN_EG * 1055 / 1000,
    2 * PAWN_EG * 1042 / 1000,
    3 * PAWN_EG * 963 / 1000,
    4 * PAWN_EG * 1038 / 1000,
    5 * PAWN_EG * 950 / 1000,
    6 * PAWN_EG * 930 / 1000,
];

// used at startup to use lookup tables
#[cold]
pub fn init() {
    for imp in 0..2 {
        for d in 1..64 {
            for mc in 1..64 {
                let r: f64 = (d as f64).log(2.0) * (mc as f64).log(2.0) / 1.95;
                unsafe {
                    REDUCTIONS[0][imp][d][mc] = r as i16;
                    REDUCTIONS[1][imp][d][mc] = (REDUCTIONS[0][imp][d][mc] - 1).max(1);

                    // Increase reduction for non-PV nodes when eval is not improving
                    if imp == 0 && r > 1.0 {
                        REDUCTIONS[0][imp][d][mc] += 1;
                    }
                }
            }
        }
    }

    for d in 0..16 {
        unsafe {
            FUTILITY_MOVE_COUNTS[0][d] = (2.4 + 0.74 * (d as f64).powf(1.78)) as i32;
            FUTILITY_MOVE_COUNTS[1][d] = (5.0 + 1.0 * (d as f64).powf(2.0)) as i32;
        }
    }
}

pub struct Stack {
    pv: BitMove,
    cont_history: *mut PieceToHistory,
    ply: u16,
    current_move: BitMove,
    excluded_move: BitMove,
    killers: [BitMove; 2],
    static_eval: Value,
    stat_score: i32,
    move_count: u32,
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

/// A Stack for the searcher, with information being contained per-ply.
pub struct ThreadStack {
    stack: [Stack; THREAD_STACK_SIZE],
}

impl ThreadStack {
    pub fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    /// Gets a certain frame from the stack.
    ///
    /// Assumes the frame is within bounds, otherwise undefined behavior.
    pub fn get(&mut self, frame: usize) -> &mut Stack {
        debug_assert!(frame < THREAD_STACK_SIZE);
        unsafe { self.stack.get_unchecked_mut(frame) }
    }

    /// Get the ply at Zero
    pub fn ply_zero(&mut self) -> &mut Stack {
        self.get(4)
    }
}

pub struct Searcher {
    // Synchronization primitives
    pub id: usize,
    pub kill: AtomicBool,
    pub searching: Arc<GuardedBool>,
    pub cond: Arc<LockLatch>,

    // search data
    pub depth_completed: i16,
    pub limit: Limits,
    pub board: Board,
    pub time_man: &'static TimeManager,
    pub pawns: PawnTable,
    pub material: Material,
    pub root_moves: UnsafeCell<RootMoveList>,
    pub selected_depth: i16,
    pub last_best_move: BitMove,
    pub last_best_move_depth: i16,
    pub nodes: AtomicU64,

    pub counter_moves: CounterMoveHistory,
    pub main_history: ButterflyHistory,
    pub capture_history: CapturePieceToHistory,
    pub cont_history: ContinuationHistory,

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
            board: Board::start_pos(),
            time_man: timer(),
            pawns: PawnTable::new(),
            material: Material::new(),
            root_moves: UnsafeCell::new(RootMoveList::new()),
            selected_depth: 0,
            last_best_move: BitMove::null(),
            last_best_move_depth: 0,
            nodes: AtomicU64::new(0),
            counter_moves: CounterMoveHistory::new(),
            main_history: ButterflyHistory::new(),
            capture_history: CapturePieceToHistory::new(),
            cont_history: ContinuationHistory::new(),
            previous_score: 0,
            best_move: BitMove::null(),
            failed_low: false,
            best_move_changes: 0.0,
            previous_time_reduction: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.pawns.clear();
        self.material.clear();
        self.previous_time_reduction = 0.0;
        self.previous_score = INFINITE;
        self.counter_moves.clear();
        self.main_history.clear();
        self.capture_history.clear();
        self.cont_history.clear();
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

        // Increment the TT search table.
        tt().new_search();
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
        if !self.limit.limits_type.is_depth() {
            let mut best_thread: &Searcher = &self;
            threadpool()
                .threads
                .iter()
                .map(|u| unsafe { &**u.get() })
                .for_each(|th| {
                    let depth_diff = th.depth_completed as i32 - best_thread.depth_completed as i32;
                    let score_diff =
                        th.root_moves().first().score - best_thread.root_moves().first().score;
                    if score_diff > 0 && depth_diff >= 0 {
                        best_thread = th;
                    }
                });
            best_move = best_thread.root_moves().first().bit_move;
            best_score = best_thread.root_moves().first().score;

            // Cases where the MainTHread did not have the correct best move, display it.
            if self.use_stdout() && best_thread.id != self.id {
                best_thread.pv(best_thread.depth_completed, NEG_INFINITE, INFINITE);
            }
        }

        self.previous_score = best_score;
        self.best_move = best_move;

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

        let mut stack: ThreadStack = ThreadStack::new();

        for i in [0, 1, 2, 3, 4].iter() {
            stack.get(*i).cont_history = &mut self.cont_history[(Piece::None, SQ(0))] as *mut _;
        }

        // If use a max_depth limit, use that as the max depth.
        let max_depth = if self.main_thread() {
            if let LimitsType::Depth(d) = self.limit.limits_type {
                d as i16
            } else {
                MAX_PLY as i16
            }
        } else {
            MAX_PLY as i16
        };

        if self.main_thread() {
            self.best_move_changes = 0.0;
            self.failed_low = false;
        }

        // The depth to start searching at based on the thread ID.
        let start_ply: i16 = START_PLY[self.id % THREAD_DIST];
        // The number of plies to skip each iteration.
        let skip_size: i16 = SKIP_SIZE[self.id % THREAD_DIST];
        let mut depth: i16 = start_ply + 1;

        let mut delta: i32 = NEG_INFINITE as i32;
        #[allow(unused_assignments)]
        let mut best_value: i32 = NEG_INFINITE as i32;
        let mut alpha: i32 = NEG_INFINITE as i32;
        let mut beta: i32 = INFINITE as i32;

        let mut time_reduction: f64 = 1.0;

        stack.ply_zero().ply = 0;

        // Iterative deeping. Start at the base ply (determined by thread_id), and then increment
        // by the skip size after searching that depth. If searching for depth, non-main threads
        // will ignore the max_depth and instead wait for a stop signal.
        'iterative_deepening: while !self.stop() && depth < max_depth {
            if self.main_thread() {
                self.best_move_changes *= 0.440;
                self.failed_low = false;
            }

            // rollback all the root moves, ala set the previous score to the current score.
            self.root_moves().rollback();

            // Delta gives a bound in the iterative loop before re-searching that position.
            // Only applicable for a depth of 5 and beyond.
            if depth >= 5 {
                let prev_best_score = self.root_moves().first().prev_score;
                delta = 20;
                alpha = max(prev_best_score - delta, NEG_INFINITE);
                beta = min(prev_best_score + delta, INFINITE);
            }

            // Loop until we find a value that is within the bounds of alpha, beta, and the delta margin.
            'aspiration_window: loop {
                // search!
                best_value = self.search::<PV>(alpha, beta, stack.ply_zero(), depth, false, false);

                // Sort root moves based on the scores
                self.root_moves().sort();

                if self.stop() {
                    // In case of a fail high or fail low, we do not choose to sort the moves,
                    // as the resulting scores would be incorrect
                    break 'aspiration_window;
                }

                // Order root moves by the score retrieved post search.

                if self.use_stdout()
                    && self.main_thread()
                    && (best_value <= alpha || best_value >= beta)
                    && self.time_man.elapsed() > 3000
                {
                    self.pv(depth, alpha, beta);
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
            if self.use_stdout() && self.main_thread() && self.time_man.elapsed() > 6 {
                if self.stop() {
                    self.pv(depth, NEG_INFINITE, INFINITE);
                } else {
                    self.pv(depth, alpha, beta);
                }
            }

            if !self.stop() {
                self.depth_completed = depth;
            }

            let curr_best_move = unsafe { (*self.root_moves.get()).first().bit_move };

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
            if self.limit.use_time_management().is_some() {
                if !self.stop() {
                    let score_diff: i32 = best_value - self.previous_score;

                    let improving_factor: i64 = (232)
                        .max((787).min(306 + 119 * self.failed_low as i64 - 6 * score_diff as i64));

                    time_reduction = 1.0;

                    // If the bestMove is stable over several iterations, reduce time accordingly
                    for i in 3..6 {
                        if self.last_best_move_depth * i < self.depth_completed {
                            time_reduction *= 1.42;
                        }
                    }

                    // Use part of the gained time from a previous stable move for the current move
                    let mut unstable_factor: f64 = 1.0 + self.best_move_changes;
                    unstable_factor *= self.previous_time_reduction.powf(0.40) / time_reduction;

                    // Stop the search if we have only one legal move, or if available time elapsed
                    if self.root_moves().len() == 1
                        || self.time_man.elapsed()
                            >= (self.time_man.ideal_time() as f64
                                * unstable_factor as f64
                                * improving_factor as f64
                                / 600.0) as i64
                    {
                        threadpool().set_stop(true);
                        break 'iterative_deepening;
                    }
                }
            }
        }

        if self.main_thread() {
            self.previous_time_reduction = time_reduction;
        }
    }

    // The searching function for a specific depth.
    fn search<N: PVNode>(
        &mut self,
        mut alpha: i32,
        mut beta: i32,
        ss: &mut Stack,
        depth: i16,
        cut_node: bool,
        skip_early_pruning: bool,
    ) -> i32 {
        if self.board.fifty_move_rule() || self.board.threefold_repetition() {
            return DRAW as i32;
        }

        if depth < 1 {
            return self.qsearch::<N>(alpha, beta, ss, 0);
        }

        assert!(depth >= 1);
        assert!(depth < MAX_PLY as i16);
        let is_pv: bool = N::is_pv();
        let ply: u16 = ss.ply;
        let at_root: bool = ply == 0;
        let zob: u64;
        let in_check: bool = self.board.in_check();

        let mut extension: i16;
        let mut new_depth: i16;

        let mut best_move: BitMove;
        let excluded_move: BitMove;

        let mut value: Value = NEG_INFINITE;
        let mut best_value: Value = NEG_INFINITE;
        let mut moves_played: u32 = 0;
        let mut pos_eval: i32;

        ss.move_count = 0;
        let mut moved_piece: Piece;

        let mut captures_searched: [BitMove; 32] = [BitMove::null(); 32];
        let mut captures_count = 0;
        let mut quiets_searched: [BitMove; 64] = [BitMove::null(); 64];
        let mut quiets_count = 0;

        let mut capture_or_promotion: bool;
        let mut gives_check: bool;
        let mut tt_capture: bool;
        let mut move_count_pruning: bool;
        let mut skip_quiets: bool;
        let singular_extension_node: bool;
        let improving: bool;
        let pv_exact: bool;

        // If we are the main thread, check the time.
        if self.main_thread() {
            self.check_time();
        }

        if !at_root {
            // Check for stop conditions.
            if self.stop() || ply >= MAX_PLY {
                if !in_check && ply >= MAX_PLY {
                    return self.eval();
                } else {
                    return ZERO;
                }
            }

            // Mate distance pruning. This ensures that checkmates closer to the root
            // have a higher value than otherwise.
            alpha = alpha.max(mated_in(ply));
            beta = beta.min(mate_in(ply + 1));
            if alpha >= beta {
                return alpha;
            }
        }

        // increment the next ply
        ss.incr().ply = ply + 1;
        // Set the killer moves two plies in advance to be nothing.
        ss.offset(2).killers = [BitMove::null(); 2];
        best_move = BitMove::null();
        ss.current_move = BitMove::null();
        ss.offset(1).excluded_move = BitMove::null();
        ss.cont_history = &mut self.cont_history[(Piece::None, SQ(0))] as *mut _;

        // square the previous piece moved.
        let prev_sq: SQ = ss.offset(-1).current_move.get_src();

        // Initialize statScore to zero for the grandchildren of the current position.
        // So statScore is shared between all grandchildren and only the first grandchild
        // starts with statScore = 0. Later grandchildren start with the last calculated
        // statScore of the previous grandchild. This influences the reduction rules in
        // LMR which are based on the statScore of parent position.
        ss.offset(-2).stat_score = 0;

        // probe the transposition table
        excluded_move = ss.excluded_move;
        zob = self.board.zobrist() ^ (excluded_move.get_raw() as u64).wrapping_shl(16);
        let (tt_hit, tt_entry): (bool, &mut Entry) = tt().probe(zob);
        let tt_value: Value = if tt_hit {
            value_from_tt(tt_entry.score, ss.ply)
        } else {
            NONE
        };
        let tt_move: BitMove = if at_root {
            self.root_moves().first().bit_move
        } else if tt_hit {
            tt_entry.best_move
        } else {
            BitMove::null()
        };

        // At non-PV nodes, check for a better TT value to return.
        if !is_pv
            && tt_hit
            && tt_entry.depth as i16 >= depth as i16
            && tt_value != NONE
            && correct_bound_eq(tt_value, beta, tt_entry.node_type())
        {
            if tt_move != BitMove::null() {
                if tt_value >= beta {
                    if !self.board.is_capture_or_promotion(tt_move) {
                        self.update_quiet_stats(
                            tt_move,
                            ss,
                            &quiets_searched[0..0],
                            stat_bonus(depth),
                        );
                    }

                    // Extra penalty for a quiet TT move in previous ply when it gets refuted
                    if ss.offset(-1).move_count == 1
                        && self.board.piece_captured_last_turn().is_none()
                    {
                        let piece_at_sq = self.board.piece_at_sq(prev_sq);
                        self.update_continuation_histories(
                            ss.offset(-1),
                            piece_at_sq,
                            prev_sq,
                            -stat_bonus(depth + 1),
                        );
                    }
                } else if !self.board.is_capture_or_promotion(tt_move) {
                    // Penalty for a quiet ttMove that fails low
                    let penalty = -stat_bonus(depth);
                    let turn = self.board.turn();
                    let moved_piece = self.board.moved_piece(tt_move);
                    self.main_history.update((turn, tt_move), penalty);
                    self.update_continuation_histories(
                        ss,
                        moved_piece,
                        tt_move.get_dest(),
                        penalty,
                    );
                }
            }

            return tt_value;
        }

        // Get and set the position eval
        if in_check {
            // A checking position should never be evaluated. We go directly to the moves loop
            // now.
            ss.static_eval = NONE;
            pos_eval = NONE;
            improving = false;
        } else {
            if tt_hit {
                pos_eval = if tt_entry.eval as i32 == NONE {
                    self.eval()
                } else {
                    tt_entry.eval as i32
                };
                ss.static_eval = pos_eval;

                // check for tt value being a better position evaluation
                if tt_value != NONE && correct_bound(tt_value, pos_eval, tt_entry.node_type()) {
                    pos_eval = tt_value;
                }
            } else {
                pos_eval = self.eval();
                ss.static_eval = pos_eval;
                // Place the evaluation into the tt, as it's otherwise empty
                tt_entry.place(
                    zob,
                    BitMove::null(),
                    NONE as i16,
                    pos_eval as i16,
                    -6,
                    NodeBound::NoBound,
                    tt().time_age(),
                );
            }

            improving = {
                let p_ss = ss.offset(-2).static_eval;
                ss.static_eval >= p_ss || p_ss == NONE
            };
        }

        if !in_check && !skip_early_pruning && self.board.non_pawn_material_all() != 0 {
            // Razoring. At the lowest depth before qsearch, If the evaluation + a margin still
            // isn't better than alpha, go straight to qsearch.
            if !is_pv && depth < 3 && pos_eval <= alpha - RAZOR_MARGIN[depth as usize] {
                let r_alpha = alpha - (depth >= 2) as i32 * RAZOR_MARGIN[depth as usize];
                let v = self.qsearch::<NonPV>(r_alpha, r_alpha + 1, ss, 0);
                if depth < 2 || v <= r_alpha {
                    return v;
                }
            }

            // Futility Pruning. Disregard moves that have little chance of raising the callee's
            // alpha value. Rather, return the position evaluation as an estimate for the current
            // move's strength
            if !at_root
                && depth < 7
                && pos_eval - futility_margin(depth, improving) >= beta
                && pos_eval < 10000
            {
                return pos_eval;
            }
        }

        // Continuation histories of the previous moved from 1, 2, and 4 moves ago.
        let cont_hists = [
            ss.offset(-1).cont_history,
            ss.offset(-2).cont_history,
            ptr::null(),
            ss.offset(-4).cont_history,
        ];

        let counter: BitMove = self.counter_moves[(self.board.piece_at_sq(prev_sq), prev_sq)];
        let mut move_picker = MovePicker::main_search(
            &self.board,
            depth as i16,
            &self.main_history,
            &self.capture_history,
            &cont_hists as *const _,
            tt_move,
            ss.killers,
            counter,
        );

        singular_extension_node = !at_root
            && depth >= 8
            && tt_move != BitMove::null()
            && tt_value != NONE
            && excluded_move == BitMove::null()
            && (tt_entry.node_type() as u8 & NodeBound::LowerBound as u8) != 0
            && tt_entry.depth as i16 >= depth - 3;

        skip_quiets = false;
        tt_capture = false;
        pv_exact = is_pv && tt_entry.node_type() == NodeBound::Exact;
        while let Some(mov) = move_picker.next(skip_quiets) {
            if mov == excluded_move {
                continue;
            }

            moves_played += 1;
            ss.move_count = moves_played;

            extension = 0;
            gives_check = self.board.gives_check(mov);
            capture_or_promotion = self.board.is_capture_or_promotion(mov);
            moved_piece = self.board.moved_piece(mov);

            move_count_pruning = depth < 16
                && moves_played as i32
                    > unsafe { FUTILITY_MOVE_COUNTS[improving as usize][depth as usize] };

            // Singular extension search. If all moves but one fail low on a search
            // of (alpha-s, beta-s), and just one fails high on (alpha, beta), then
            // that move is singular and should be extended. To verify this we do a
            // reduced search on on all the other moves but the ttMove and if the
            // result is lower than ttValue minus a margin then we will extend the ttMove.
            if singular_extension_node && mov == tt_move && self.board.legal_move(mov) {
                let rbeta: i32 = (-MATE).max(tt_value - 2 * depth as i32);
                ss.excluded_move = mov;
                value = self.search::<NonPV>(rbeta - 1, rbeta, ss, depth / 2, cut_node, true);
                ss.excluded_move = BitMove::null();
                if value < rbeta {
                    extension = 1;
                }
            } else if gives_check && !move_count_pruning && self.board.see_ge(mov, 0) {
                extension = 1;
            }

            new_depth = depth - 1 + extension;

            // Pruning at a shallow depth
            if !at_root
                && self.board.non_pawn_material(self.board.turn()) != 0
                && best_value > MATED_IN_MAX_PLY
            {
                if !capture_or_promotion
                    && !gives_check
                    && (!self.board.advanced_pawn_push(mov)
                        || self.board.non_pawn_material_all() >= 5000)
                {
                    if move_count_pruning {
                        skip_quiets = true;
                        continue;
                    }

                    // Reduced depth of the next LMR search
                    let lmr_depth: i16 =
                        (new_depth - reduction::<PV>(improving, depth, moves_played)).max(0);

                    // Countermoves based pruning
                    unsafe {
                        if lmr_depth < 3
                            && (*cont_hists[0])[(moved_piece, mov.get_dest())] < 0
                            && (*cont_hists[1])[(moved_piece, mov.get_dest())] < 0
                        {
                            continue;
                        }
                    }

                    // Futility pruning: parent node
                    if lmr_depth < 7
                        && !in_check
                        && ss.static_eval + 256 + 200 * lmr_depth as i32 <= alpha
                    {
                        continue;
                    }

                    // Prune moves with negative SEE
                    if lmr_depth < 8
                        && !self
                            .board
                            .see_ge(mov, -35 * lmr_depth as i32 * lmr_depth as i32)
                    {
                        continue;
                    }
                } else if depth < 7
                    && extension == 0
                    && !self
                        .board
                        .see_ge(mov, -CAPTURE_PRUNE_MARGIN[depth as usize])
                {
                    continue;
                }
            }

            // speculative prefetch for the next key.
            tt().prefetch(self.board.key_after(mov));

            if !self.board.legal_move(mov) {
                ss.move_count -= 1;
                moves_played -= 1;
                continue;
            }

            if mov == tt_move && capture_or_promotion {
                tt_capture = true;
            }
            ss.current_move = mov;
            ss.cont_history = &mut self.cont_history[(moved_piece, mov.get_dest())] as *mut _;

            // do the move
            self.apply_move(mov, gives_check);

            // prefetch next TT entry
            tt().prefetch(self.board.zobrist());

            // At higher depths, do a search of a lower ply to see if this move is
            // worth searching. We don't do this for capturing or promotion moves.
            let do_full_depth: bool = if moves_played > 1
                && depth >= 3
                && (!capture_or_promotion || move_count_pruning)
            {
                let mut r: i16 = reduction::<PV>(improving, depth, moves_played);

                if capture_or_promotion {
                    r = (r - 1).max(0);
                } else {
                    // Decrease reduction if opponent's move count is high
                    if ss.offset(-1).move_count > 15 {
                        r -= 1;
                    }

                    if pv_exact {
                        r -= 1;
                    }

                    if tt_capture {
                        r += 1;
                    }

                    // Decrease reduction for moves that escape a capture. Filter out
                    // castling moves, because they are coded as "king captures rook" and
                    // hence break make_move().
                    if mov.move_type() == MoveType::Normal
                        && !self
                            .board
                            .see_ge(BitMove::make(0, mov.get_dest(), mov.get_src()), 0)
                    {
                        r -= 2;
                    }

                    ss.stat_score = unsafe {
                        self.main_history[(!self.board.turn(), mov)] as i32
                            + (*cont_hists[0])[(moved_piece, mov.get_dest())] as i32
                            + (*cont_hists[1])[(moved_piece, mov.get_dest())] as i32
                            + (*cont_hists[3])[(moved_piece, mov.get_dest())] as i32
                            - 4000
                    };

                    // Decrease/increase reduction by comparing opponent's stat score
                    if ss.stat_score >= 0 && ss.offset(-1).stat_score < 0 {
                        r -= 1;
                    } else if ss.offset(-1).stat_score >= 0 && ss.stat_score < 0 {
                        r += 1;
                    }

                    r = (r - (ss.stat_score / 20000) as i16).max(0) as i16;
                }

                let d = (new_depth - r).max(1);

                value = -self.search::<NonPV>(-(alpha + 1), -alpha, ss.incr(), d, true, false);
                value > alpha && d != new_depth
            } else {
                !is_pv || moves_played > 1
            };

            // If the value is potentially better, do a full depth search.
            if do_full_depth {
                value = -self.search::<NonPV>(
                    -(alpha + 1),
                    -alpha,
                    ss.incr(),
                    new_depth,
                    !cut_node,
                    false,
                );
            }

            // If on the PV node and the node might be a continuation, search for a full depth
            // with a PV value.
            if is_pv && (moves_played == 1 || (value > alpha && (at_root || value < beta))) {
                value = -self.search::<PV>(-beta, -alpha, ss.incr(), new_depth, false, false);
            }

            self.board.undo_move();
            assert!(value > NEG_INFINITE);
            assert!(value < INFINITE);

            // Unsafe to return any other value here when the threads are stopped.
            if self.stop() {
                return ZERO;
            }

            if at_root {
                let mut incr_bmc: bool = false;
                {
                    let rm: &mut RootMove = self.root_moves().find(mov).unwrap();

                    // Insert the score into the RootMoves list
                    if moves_played == 1 || value > alpha {
                        rm.depth_reached = depth;
                        rm.score = value;
                        if moves_played > 1 && self.main_thread() && depth > 5 {
                            incr_bmc = true;
                        }
                    } else {
                        rm.score = NEG_INFINITE;
                    }
                }
                // If we have a new best move at root, update the nmber of best_move changes.
                if incr_bmc {
                    self.best_move_changes += 1.0;
                }
            }

            if value > best_value {
                best_value = value;

                if value > alpha {
                    best_move = mov;

                    if is_pv && !at_root {
                        ss.incr().pv = mov;
                    }

                    if is_pv && value < beta {
                        alpha = value;
                    } else {
                        break;
                    }
                }
            }

            // If best_move wasn't found, add it to the list of quiets / captures that failed
            if mov != best_move {
                if capture_or_promotion && captures_count < 32 {
                    captures_searched[captures_count] = mov;
                    captures_count += 1;
                } else if !capture_or_promotion && quiets_count < 64 {
                    quiets_searched[quiets_count] = mov;
                    quiets_count += 1;
                }
            }
        }

        // check for checkmate or draw.
        if moves_played == 0 {
            if excluded_move != BitMove::null() {
                return alpha;
            } else if in_check {
                return mated_in(ss.ply);
            } else {
                return DRAW as i32;
            }
        } else if best_move != BitMove::null() {
            // If the best move is quiet, update move heuristics
            if !self.board.is_capture_or_promotion(best_move) {
                self.update_quiet_stats(
                    best_move,
                    ss,
                    &quiets_searched[0..quiets_count],
                    stat_bonus(depth),
                );
            } else {
                self.update_capture_stats(
                    best_move,
                    &captures_searched[0..captures_count],
                    stat_bonus(depth),
                );
            }

            // penalize quiet TT move that was refuted.
            if ss.offset(-1).move_count == 1 && self.board.piece_captured_last_turn().is_none() {
                let piece_at_sq = self.board.piece_at_sq(prev_sq);
                self.update_continuation_histories(
                    ss.offset(-1),
                    piece_at_sq,
                    prev_sq,
                    -stat_bonus(depth + 1),
                );
            }
        } else if depth >= 3
            && self.board.piece_captured_last_turn().is_none()
            && ss.offset(-1).current_move.is_okay()
        {
            let piece_at_sq = self.board.piece_at_sq(prev_sq);
            // bonus for counter move that failed low
            self.update_continuation_histories(
                ss.offset(-1),
                piece_at_sq,
                prev_sq,
                stat_bonus(depth),
            );
        }

        let node_bound = if best_value as i32 >= beta {
            NodeBound::LowerBound
        } else if is_pv && !best_move.is_null() {
            NodeBound::Exact
        } else {
            NodeBound::UpperBound
        };

        if excluded_move != BitMove::null() {
            tt_entry.place(
                zob,
                best_move,
                value_to_tt(best_value, ss.ply),
                ss.static_eval as i16,
                depth as i16,
                node_bound,
                tt().time_age(),
            );
        }

        best_value
    }

    /// Called by the main search when the depth limit has been reached. This function only traverses capturing moves
    /// and possible checking moves, unless its in check.
    ///
    /// Depth must be less than or equal to zero,
    fn qsearch<N: PVNode>(
        &mut self,
        mut alpha: i32,
        beta: i32,
        ss: &mut Stack,
        rev_depth: i16,
    ) -> i32 {
        if self.board.fifty_move_rule() || self.board.threefold_repetition() {
            return DRAW as i32;
        }

        let is_pv: bool = N::is_pv();

        assert!(alpha >= NEG_INFINITE);
        assert!(beta <= INFINITE);
        assert!(alpha < beta);
        assert!(rev_depth <= 0);
        assert!(is_pv || (alpha == beta - 1));

        let ply: u16 = ss.ply;
        let zob: u64 = self.board.zobrist();

        let mut value: Value;
        let mut best_value: Value;
        let pos_eval: Value;
        let futility_base: Value;
        let mut futility_value: Value;
        let mut evasion_prunable: bool;
        let mut moves_played: u32 = 0;
        let old_alpha = alpha;

        let in_check: bool = self.board.in_check();

        if ply >= MAX_PLY {
            if !in_check {
                return self.eval();
            } else {
                return ZERO;
            }
        }

        let (tt_hit, tt_entry): (bool, &mut Entry) = tt().probe(zob);
        let tt_value: Value = if tt_hit {
            value_from_tt(tt_entry.score, ss.ply)
        } else {
            NONE
        };

        // Determine whether or not to include checking moves.
        let tt_depth: i16 = if in_check || rev_depth >= 0 { 0 } else { -1 };

        // increment the next ply
        ss.incr().ply = ply + 1;
        ss.current_move = BitMove::null();
        let tt_move = if tt_hit {
            tt_entry.best_move
        } else {
            BitMove::null()
        };
        let mut best_move = tt_move;

        if !is_pv
            && tt_hit
            && tt_entry.depth as i16 >= tt_depth
            && tt_value != NONE
            && correct_bound_eq(tt_value, beta, tt_entry.node_type())
        {
            return tt_value;
        }

        // Evaluate the position statically.
        if in_check {
            ss.static_eval = NONE;
            best_value = NEG_INFINITE;
            futility_base = NEG_INFINITE;
        } else {
            if tt_hit {
                if tt_entry.eval as i32 == NONE {
                    best_value = self.eval();
                    pos_eval = best_value;
                    ss.static_eval = best_value;
                } else {
                    best_value = tt_entry.eval as i32;
                    pos_eval = best_value;
                    ss.static_eval = best_value;
                }

                if tt_value != NONE && correct_bound(tt_value, best_value, tt_entry.node_type()) {
                    best_value = tt_value;
                }
            } else {
                best_value = self.eval();
                pos_eval = best_value;
                ss.static_eval = best_value;
            }

            if best_value >= beta {
                if !tt_hit {
                    tt_entry.place(
                        zob,
                        BitMove::null(),
                        value_to_tt(best_value, ss.ply),
                        pos_eval as i16,
                        -6,
                        NodeBound::LowerBound,
                        tt().time_age(),
                    );
                }
                return best_value;
            }

            if is_pv && best_value > alpha {
                alpha = best_value;
            }

            futility_base = 128 + best_value;
        }

        let recap_sq = ss.offset(-1).current_move.get_dest();
        let mut move_picker = MovePicker::qsearch(
            &self.board,
            rev_depth,
            tt_move,
            &self.main_history,
            &self.capture_history,
            recap_sq,
        );

        while let Some(mov) = move_picker.next(false) {
            let gives_check: bool = self.board.gives_check(mov);
            moves_played += 1;

            // futility pruning
            if !in_check
                && !gives_check
                && futility_base > -10000
                && !self.board.advanced_pawn_push(mov)
            {
                let piece_at = self.board.piece_at_sq(mov.get_src()).type_of();
                futility_value = futility_base + piecetype_value(piece_at, true);

                if futility_value <= alpha {
                    best_value = best_value.max(futility_value);
                    continue;
                }

                if futility_base <= alpha && !self.board.see_ge(mov, 1) {
                    best_value = best_value.max(futility_base);
                    continue;
                }
            }

            evasion_prunable = in_check
                && (rev_depth != 0 || moves_played > 2)
                && best_value > MATED_IN_MAX_PLY
                && !self.board.is_capture(mov);

            // Don't search moves that lead to a negative static exchange
            if (!in_check || evasion_prunable) && !self.board.see_ge(mov, 0) {
                continue;
            }

            tt().prefetch(self.board.key_after(mov));

            if !self.board.legal_move(mov) {
                moves_played -= 1;
                continue;
            }

            ss.current_move = mov;
            self.apply_move(mov, gives_check);

            // prefetch next TT entry
            tt().prefetch(self.board.zobrist());

            assert_eq!(gives_check, self.board.in_check());

            value = -self.qsearch::<N>(-beta, -alpha, ss.incr(), rev_depth - 1);

            self.board.undo_move();

            assert!(value > NEG_INFINITE);
            assert!(value < INFINITE);

            // Check for new best value
            if value > best_value {
                best_value = value;

                if value > alpha {
                    if is_pv {
                        ss.incr().pv = best_move;
                    }
                    if is_pv && value < beta {
                        best_move = mov;
                        alpha = value;
                    } else {
                        tt_entry.place(
                            zob,
                            mov,
                            value_to_tt(best_value, ss.ply),
                            ss.static_eval as i16,
                            tt_depth as i16,
                            NodeBound::LowerBound,
                            tt().time_age(),
                        );
                        return value;
                    }
                }
            }
        }

        // If in checkmate, return so
        if in_check && best_value == NEG_INFINITE {
            return mated_in(ss.ply);
        }

        let node_bound = if is_pv && best_value > old_alpha {
            NodeBound::Exact
        } else {
            NodeBound::UpperBound
        };

        tt_entry.place(
            zob,
            best_move,
            value_to_tt(best_value, ss.ply),
            ss.static_eval as i16,
            tt_depth,
            node_bound,
            tt().time_age(),
        );

        assert!(best_value > NEG_INFINITE);
        assert!(best_value < INFINITE);
        best_value
    }

    /// If a new capturing best move is found, updating sorting heuristics.
    fn update_capture_stats(&mut self, mov: BitMove, captures: &[BitMove], bonus: i32) {
        let cap_hist: &mut CapturePieceToHistory = &mut self.capture_history;
        let mut moved_piece: Piece = self.board.moved_piece(mov);
        let mut captured: PieceType = self.board.captured_piece(mov);
        cap_hist.update((moved_piece, mov.get_dest(), captured), bonus);

        for m in captures.iter() {
            moved_piece = self.board.moved_piece(*m);
            captured = self.board.captured_piece(*m);
            cap_hist.update((moved_piece, m.get_dest(), captured), -bonus);
        }
    }

    /// If a new quiet best move is found, updating sorting heuristics.
    fn update_quiet_stats(&mut self, mov: BitMove, ss: &mut Stack, quiets: &[BitMove], bonus: i32) {
        if ss.killers[0] != mov {
            ss.killers[1] = ss.killers[0];
            ss.killers[0] = mov;
        }

        let us: Player = self.board.turn();
        let moved_piece = self.board.moved_piece(mov);
        let to_sq = mov.get_dest();
        self.main_history.update((us, mov), bonus);
        self.update_continuation_histories(ss, moved_piece, to_sq, bonus);

        {
            let ss_bef: &mut Stack = ss.offset(-1);
            if ss_bef.current_move.is_okay() {
                let prev_sq = ss_bef.current_move.get_dest();
                let piece = self.board.piece_at_sq(prev_sq);
                self.counter_moves[(piece, prev_sq)] = mov;
            }
        }

        for q_mov in quiets.iter() {
            self.main_history.update((us, *q_mov), -bonus);
            let q_moved_piece = self.board.moved_piece(*q_mov);
            let to_sq = q_mov.get_dest();
            self.update_continuation_histories(ss, q_moved_piece, to_sq, -bonus);
        }
    }

    // updates histories of the move pairs formed by the current move of one, two, and four
    // moves ago
    fn update_continuation_histories(&mut self, ss: &mut Stack, piece: Piece, to: SQ, bonus: i32) {
        for i in [1, 2, 4].iter() {
            let i_ss: &mut Stack = ss.offset(-i as isize);
            if i_ss.current_move.is_okay() {
                unsafe {
                    let cont_his: &mut PieceToHistory = &mut *i_ss.cont_history;
                    cont_his.update((piece, to), bonus);
                }
            }
        }
    }

    #[inline(always)]
    fn apply_move(&mut self, mov: BitMove, gives_check: bool) {
        self.nodes.fetch_add(1, Ordering::Relaxed);
        self.board
            .apply_move_pft_chk(mov, gives_check, &self.pawns, &self.material);
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
            && self.time_man.elapsed() >= self.time_man.maximum_time()
        {
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

    #[inline]
    pub fn root_moves(&self) -> &mut RootMoveList {
        unsafe { &mut *self.root_moves.get() }
    }

    /// Useful information to tell to the GUI
    fn pv(&self, depth: i16, alpha: i32, beta: i32) {
        let root_move: &RootMove = self.root_moves().first();
        let elapsed = self.time_man.elapsed() as u64;
        let nodes = threadpool().nodes();
        let mut score = if root_move.score == NEG_INFINITE {
            root_move.prev_score
        } else {
            root_move.score
        };

        if score == NEG_INFINITE {
            return;
        }

        let mut s = String::from("info");
        s.push_str(&format!(" depth {}", depth));
        if score.abs() < MATE - MAX_PLY as i32 {
            score *= 100;
            score /= PAWN_EG;
            s.push_str(&format!(" score cp {}", score));
        } else {
            let mut mate_in = if score > 0 {
                MATE - score + 1
            } else {
                -MATE - score
            };
            mate_in /= 2;
            s.push_str(&format!(" score mate {}", mate_in));
        }
        if root_move.score >= beta {
            s.push_str(" lowerbound");
        } else if root_move.score <= alpha {
            s.push_str(" upperbound");
        }
        s.push_str(&format!(" nodes {}", nodes));
        if elapsed > 1000 {
            s.push_str(&format!(" nps {}", (nodes * 1000) / elapsed));
            s.push_str(&format!(" hashfull {:.2}", tt().hash_percent()));
        }
        s.push_str(&format!(" time {}", elapsed));
        s.push_str(&format!(" pv {}", root_move.bit_move.to_string()));
        println!("{}", s);
    }
}

impl Drop for Searcher {
    fn drop(&mut self) {
        self.searching.set(false);
    }
}

fn correct_bound_eq(tt_value: i32, beta: i32, bound: NodeBound) -> bool {
    if tt_value >= beta {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}

fn correct_bound(tt_value: i32, val: i32, bound: NodeBound) -> bool {
    if tt_value >= val {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}

fn mate_in(ply: u16) -> i32 {
    MATE - ply as i32
}

fn mated_in(ply: u16) -> i32 {
    -MATE + ply as i32
}

fn value_to_tt(value: i32, ply: u16) -> i16 {
    if value >= MATE_IN_MAX_PLY {
        (value + ply as i32) as i16
    } else if value <= MATED_IN_MAX_PLY {
        (value - ply as i32) as i16
    } else {
        value as i16
    }
}

fn value_from_tt(value: i16, ply: u16) -> i32 {
    if value as i32 == NONE {
        NONE
    } else if value as i32 >= MATE_IN_MAX_PLY {
        value as i32 - ply as i32
    } else if value as i32 <= MATED_IN_MAX_PLY {
        value as i32 + ply as i32
    } else {
        value as i32
    }
}

#[inline]
fn futility_margin(depth: i16, improving: bool) -> i32 {
    depth as i32 * (175 - 50 * improving as i32)
}

fn reduction<PV: PVNode>(i: bool, depth: i16, mn: u32) -> i16 {
    unsafe {
        REDUCTIONS[PV::is_pv() as usize][i as usize][(depth as usize).min(63)]
            [(mn as usize).min(63)]
    }
}

fn stat_bonus(depth: i16) -> i32 {
    if depth > 17 {
        0
    } else {
        let d = depth as i32;
        d * d + 2 * d - 2
    }
}

#[test]
fn how_big() {
    let x = mem::size_of::<Searcher>() / 1000;
    println!("size of searcher: {} KB", x);
}
