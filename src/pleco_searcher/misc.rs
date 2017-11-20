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
    pub score: i32,
    pub prev_score: i32,
    pub depth_reached: u16
}

// Moves with higher score for a higher depth are less
impl Ord for RootMove {
    fn cmp(&self, other: &RootMove) -> CmpOrder {
        let value_diff = self.score - other.score;
        if value_diff == 0 {
            let prev_value_diff = self.prev_score as i32 - other.prev_score as i32;
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
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move: bit_move,
            score: NEG_INFINITY as i32,
            prev_score: NEG_INFINITY as i32,
            depth_reached: 0
        }
    }

    pub fn rollback_insert(&mut self, score: i32, depth: u16) {
        self.prev_score = self.score;
        self.score = score;
        self.depth_reached = depth;
    }

    pub fn insert(&mut self, score: i32, depth: u16) {
        self.score = score;
        self.depth_reached = depth;
    }

    pub fn rollback(&mut self) {
        self.prev_score = self.score;
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