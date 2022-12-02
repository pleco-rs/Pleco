//! This module contains the basic RootMove structures, allowing for storage of the moves from a specific position
//! alongside information about each of the moves.

pub mod root_moves_list;

use std::cmp::Ordering as CmpOrder;

use pleco::core::score::*;
use pleco::BitMove;

// 250 as this fits into 64 byte cache lines easily.
const MAX_MOVES: usize = 250;

/// Keeps track of information of a move for the position to be searched.
#[derive(Copy, Clone, Eq)]
pub struct RootMove {
    pub score: i32,
    pub prev_score: i32,
    pub bit_move: BitMove,
    pub depth_reached: i16,
}

impl RootMove {
    /// Creates a new `RootMove`.
    #[inline]
    pub fn new(bit_move: BitMove) -> Self {
        RootMove {
            bit_move,
            score: NEG_INFINITE as i32,
            prev_score: NEG_INFINITE as i32,
            depth_reached: 0,
        }
    }

    /// Places the current score into the previous_score field, and then updates
    /// the score and depth.
    #[inline]
    pub fn rollback_insert(&mut self, score: i32, depth: i16) {
        self.prev_score = self.score;
        self.score = score;
        self.depth_reached = depth;
    }

    /// Inserts a score and depth.
    #[inline]
    pub fn insert(&mut self, score: i32, depth: i16) {
        self.score = score;
        self.depth_reached = depth;
    }

    /// Places the current score in the previous score.
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
            return CmpOrder::Less;
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
        self.score == other.score && self.prev_score == other.prev_score
    }
}
