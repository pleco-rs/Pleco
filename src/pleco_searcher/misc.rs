use super::THREAD_STACK_SIZE;
use board::eval::*;
use core::piece_move::BitMove;
use std::cmp::Ordering as CmpOrder;
use std::mem;

pub trait PVNode {
    fn is_pv() -> bool;
}

pub struct PV {}
pub struct NonPV {}

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
pub struct RootMove {
    pub bit_move: BitMove,
    pub score: i16,
    pub prev_score: i16,
    pub depth_reached: u16
}

// Moves with higher score for a higher depth are less
impl Ord for RootMove {
    fn cmp(&self, other: &RootMove) -> CmpOrder {
        let value_diff = self.score as i32 - other.score as i32;
        if value_diff > 0 {
            let depth_diff = self.depth_reached as i32 - other.depth_reached as i32;
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


pub struct ThreadStack {
    pub pos_eval: i16,
}

impl ThreadStack {
    pub fn new() -> Self {
        ThreadStack {
            pos_eval: 0
        }
    }
}

pub fn init_thread_stack() -> [ThreadStack; THREAD_STACK_SIZE] {
    let s: [ThreadStack; THREAD_STACK_SIZE] = unsafe { mem::zeroed() };
    s
}