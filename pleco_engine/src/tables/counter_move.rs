use std::ops::{Index,IndexMut};
use pleco::core::masks::*;
use pleco::{PieceType, SQ,Player,BitMove};

use super::StatBoard;


/// CounterMoveHistory stores counter moves indexed by [player][piece][to] of the previous
/// move
pub struct CounterMoveHistory {
    a: [[[BitMove; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}

// [Us][Piece][To SQ]
#[allow(non_camel_case_types)]
type CM_idx = (Player, PieceType, SQ);

impl Index<CM_idx> for CounterMoveHistory {
    type Output = BitMove;

    fn index(&self, idx: CM_idx) -> &Self::Output {
        unsafe {
            self.a.get_unchecked(idx.0 as usize)    // [Player Moved]
                .get_unchecked(idx.1 as usize)      // [Piece Moved]
                .get_unchecked((idx.2).0 as usize)  // [To SQ]
        }
    }
}

impl IndexMut<CM_idx> for CounterMoveHistory {
    fn index_mut(&mut self, idx: CM_idx) -> &mut Self::Output {
        unsafe {
            self.a.get_unchecked_mut(idx.0 as usize)    // [Player Moved]
                .get_unchecked_mut(idx.1 as usize)      // [Piece Moved]
                .get_unchecked_mut((idx.2).0 as usize)  // [To SQ]
        }
    }
}


impl StatBoard<BitMove, CM_idx> for CounterMoveHistory {
    const FILL: BitMove = BitMove::null();
}