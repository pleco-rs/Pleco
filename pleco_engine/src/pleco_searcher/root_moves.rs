use std::sync::{Arc,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU16,Ordering};
use std::thread::{JoinHandle,self};
use std::sync::mpsc::{channel,Receiver,Sender};

use rand::{self,Rng};

use std::{mem,time};
use std::cmp::Ordering as CmpOrder;

use pleco::board::eval::*;
use pleco::board::*;
use pleco::core::piece_move::BitMove;
use pleco::tools::tt::*;
use pleco::tools::*;
use pleco::{MoveList,Piece};

use super::thread_search::ThreadSearcher;
use super::misc::*;
use super::{TT_TABLE,THREAD_STACK_SIZE};
use super::threads::Thread;

#[derive(Copy, Clone, Eq)]
pub struct RootMove {
    pub bit_move: BitMove,
    pub score: i32,
    pub prev_score: i32,
    pub depth_reached: u16
}

// Moves with higher score for a higher depth are less
impl Ord for RootMove {
    #[inline]
    fn cmp(&self, other: &RootMove) -> CmpOrder {
        let value_diff = self.score - other.score;
        if value_diff == 0 {
            let prev_value_diff = self.prev_score - other.prev_score;
            if prev_value_diff == 0 {
                return CmpOrder::Equal;
            } else if prev_value_diff > 0 {
                return CmpOrder::Less;
            }
        } else if value_diff > 0 {
            return CmpOrder::Less
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
    #[inline]
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move: bit_move,
            score: NEG_INFINITY as i32,
            prev_score: NEG_INFINITY as i32,
            depth_reached: 0
        }
    }

    #[inline]
    pub fn rollback_insert(&mut self, score: i32, depth: u16) {
        self.prev_score = self.score;
        self.score = score;
        self.depth_reached = depth;
    }

    #[inline]
    pub fn insert(&mut self, score: i32, depth: u16) {
        self.score = score;
        self.depth_reached = depth;
    }

    #[inline]
    pub fn rollback(&mut self) {
        self.prev_score = self.score;
    }
}

#[derive(Clone)]
pub struct RootMoves {
    moves: Arc<RwLock<Vec<RootMove>>>
}

impl RootMoves {
    pub fn new() -> Self {
        RootMoves {
            moves: Arc::new(RwLock::new(Vec::new()))
        }
    }

    pub fn first(&self) -> RootMove {
        let lock = &(*self.moves.read().unwrap());
        lock.get(0).unwrap().clone()
    }

    pub fn replace(&mut self, moves: &Vec<RootMove>) {
        let mut lock = self.moves.write().unwrap();
        (*lock) = moves.clone();
    }

    pub fn to_list(&self) -> MoveList {
        let vec: Vec<BitMove> = self.moves.read().unwrap().iter().map(|m| m.bit_move).collect();
        MoveList::from(vec)
    }

    pub fn prev_best_score(&self) -> i32 {
        let lock = &(*self.moves.read().unwrap());
        lock.get(0).unwrap().prev_score
    }

    pub fn insert_score(&mut self, index: usize, score: i32) {
        let lock = &mut (*self.moves.write().unwrap());
        let mov: &mut RootMove = lock.get_mut(index).unwrap();
        mov.score = score;
    }

    pub fn insert_score_depth(&mut self, index: usize, score: i32, depth: u16) {
        let lock = &mut (*self.moves.write().unwrap());
        let mov: &mut RootMove = lock.get_mut(index).unwrap();
        mov.score = score;
        mov.depth_reached = depth;
    }

    pub fn sort(&mut self) {
        let mut moves = self.moves.write().unwrap();
        moves.sort();
    }

    pub fn rollback(&mut self) {
        let mut moves = self.moves.write().unwrap();
        for mov in (*moves).iter_mut() {
            mov.rollback()
        }
    }

    pub fn print(&self) {
        let moves = self.moves.read().unwrap();
        for mov in (*moves).iter() {
            println!(" value: {}, prev_value: {}, depth: {}, mov: {}", mov.score, mov.prev_score, mov.depth_reached, mov.bit_move);
        }
    }

    pub fn shuffle(&mut self, thread_id: usize, board: &Board) {
        if thread_id == 0 || thread_id >= 20 {
            self.moves.write().unwrap().sort_by_key(|root_move| {
                let a = root_move.bit_move;
                let piece = board.piece_at_sq((a).get_src()).unwrap();

                if a.is_capture() {
                    board.captured_piece(a).unwrap().value() - piece.value()
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
            let mut moves = self.moves.write().unwrap();
            let slice = moves.as_mut_slice();
            rand::thread_rng().shuffle(slice);
        }
    }
}

#[derive(Clone)]
pub struct ThreadInfo {
    moves: RootMoves,
    finished: Arc<AtomicBool>,
    depth_completed: Arc<AtomicU16>,
}

impl ThreadInfo {
    pub fn new(thread: &Thread) -> Self {
        ThreadInfo {
            moves: thread.root_moves.clone(),
            finished: Arc::clone(&thread.finished),
            depth_completed: Arc::clone(&thread.depth_completed)
        }
    }

    pub fn set_depth(&mut self, depth: u16) {
        self.depth_completed.store(depth, Ordering::Relaxed);
    }

    pub fn replace_rootmoves(&mut self, moves: &Vec<RootMove>) {
        self.moves.replace(moves);
    }

    pub fn best_move(&self) -> RootMove {
        self.moves.first()
    }

    pub fn best_move_and_depth(&self) -> (RootMove, u16) {
        (self.moves.first(), self.depth_completed.load(Ordering::Relaxed))
    }

    pub fn wait_for_start(&self) {
        while self.finished.load(Ordering::Relaxed) {}
    }

    pub fn wait_for_finish(&self) {
        while !self.finished.load(Ordering::Relaxed) {}
    }
}

#[derive(Clone)]
pub struct AllThreadInfo {
    all_threads: Arc<RwLock<Vec<ThreadInfo>>>
}

impl AllThreadInfo {
    pub fn new() -> Self {
        AllThreadInfo {
            all_threads: Arc::new(RwLock::new(Vec::with_capacity(8)))
        }
    }

    pub fn add(&mut self, thread_info: ThreadInfo) {
        let lock = &mut (*self.all_threads.write().unwrap());
        lock.push(thread_info);
    }

    pub fn reset_depths(&mut self) {
        let lock: &mut Vec<ThreadInfo> = &mut *self.all_threads.write().unwrap();
        lock.iter_mut().for_each(|t| t.set_depth(0));
    }

    pub fn size(&self) -> usize {
        let lock = &(*self.all_threads.read().unwrap());
        lock.len()
    }

    pub fn set_rootmoves(&mut self, board: &Board) {
        let base_moves: Vec<RootMove> = board.generate_moves()
                                             .iter()
                                             .map(|b| RootMove::new(*b))
                                             .collect();
        let all_moves = &mut (*self.all_threads.write().unwrap());

        all_moves.iter_mut()
            .for_each(|m: &mut ThreadInfo| m.replace_rootmoves(&base_moves));
    }

    pub fn wait_for_start(&self) {
        for per_thread in (*self.all_threads.read().unwrap()).iter() {
            per_thread.wait_for_start();
        }
    }

    pub fn wait_for_finish(&self) {
        for per_thread in (*self.all_threads.read().unwrap()).iter() {
            per_thread.wait_for_finish();
        }
    }

    pub fn thread_best_move(&self, thread_id: usize) -> RootMove {
        (*self.all_threads.read().unwrap()).get(thread_id).unwrap().best_move()
    }

    pub fn thread_best_move_and_depth(&self, thread_id: usize) -> (RootMove, u16) {
        let lock = &(*self.all_threads.read().unwrap());
        let thread_info: &ThreadInfo = lock.get(thread_id).unwrap();
        thread_info.best_move_and_depth()
    }

    pub fn best_rootmove(&self, use_stdout: bool) -> RootMove {
        let (mut best_root_move, mut depth_reached): (RootMove, u16) = self.thread_best_move_and_depth(0);

        for x in 1..self.size() {
            let (thread_move, thread_depth): (RootMove, u16)  = self.thread_best_move_and_depth(x);
            let depth_diff = thread_depth as i32 - depth_reached as i32;
            let value_diff = thread_move.score - best_root_move.score;
            if x != 0 {
                // If it has a bigger value and greater or equal depth
                if value_diff > 0 && depth_diff >= 0 {
                    best_root_move = thread_move;
                    depth_reached = thread_depth;
                }
            }

            if use_stdout {
                println!("id: {}, value: {}, prev_value: {}, depth: {}, depth_comp: {}, mov: {}",x, thread_move.score, best_root_move.prev_score, thread_move.depth_reached,thread_depth, thread_move.bit_move);
            }
        }
        best_root_move
    }
}