pub mod root_moves_list;
pub mod root_moves_manager;

use std::sync::{Arc,RwLock};
use std::sync::atomic::{AtomicBool,AtomicU16,Ordering};
use std::ptr::Unique;
use std::mem;
use std::heap::{Alloc, Layout, Heap};
use std::cmp::max;
use std::cell::UnsafeCell;

use rand::{self,Rng};

use std::cmp::Ordering as CmpOrder;

use pleco::board::eval::*;
use pleco::board::*;

use pleco::{MoveList,Piece,BitMove};

use super::threads::Thread;
use super::MAX_THREADS;

const MAX_MOVES: usize = 256;

#[derive(Copy, Clone,Eq)]
pub struct RootMove {
    pub score: i32,
    pub prev_score: i32,
    pub bit_move: BitMove,
    pub depth_reached: u16,
    padding: u32
}


impl RootMove {
    #[inline]
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move: bit_move,
            score: NEG_INFINITY as i32,
            prev_score: NEG_INFINITY as i32,
            depth_reached: 0,
            padding: 0
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