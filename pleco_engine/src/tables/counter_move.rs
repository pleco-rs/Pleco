use pleco::core::masks::*;
use pleco::{BitMove, Piece, SQ};
use std::ops::{Index, IndexMut};

use super::StatBoard;

/// CounterMoveHistory stores counter moves indexed by [player][piece][to] of the previous
/// move
pub struct CounterMoveHistory {
    a: [[BitMove; SQ_CNT]; PIECE_CNT],
}

// [Us][Piece][To SQ]
#[allow(non_camel_case_types)]
type CM_idx = (Piece, SQ);

impl Index<CM_idx> for CounterMoveHistory {
    type Output = BitMove;

    #[inline(always)]
    fn index(&self, idx: CM_idx) -> &Self::Output {
        unsafe {
            self.a
                .get_unchecked(idx.0 as usize) // [Piece Moved]
                .get_unchecked((idx.1).0 as usize) // [To SQ]
        }
    }
}

impl IndexMut<CM_idx> for CounterMoveHistory {
    #[inline(always)]
    fn index_mut(&mut self, idx: CM_idx) -> &mut Self::Output {
        unsafe {
            self.a
                .get_unchecked_mut(idx.0 as usize) // [Piece Moved]
                .get_unchecked_mut((idx.1).0 as usize) // [To SQ]
        }
    }
}

impl StatBoard<BitMove, CM_idx> for CounterMoveHistory {
    const FILL: BitMove = BitMove::null();
}
