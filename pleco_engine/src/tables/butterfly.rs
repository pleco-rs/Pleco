use std::ops::{Index,IndexMut};
use pleco::core::masks::*;
use pleco::{Player, BitMove};

use super::{StatBoard, NumStatBoard};

/// ButterflyBoards are 2 tables (one for each color) indexed by the move's from
/// and to squares, see chessprogramming.wikispaces.com/Butterfly+Boards
pub struct ButterflyHistory {
    a: [[i16; (SQ_CNT * SQ_CNT)]; PLAYER_CNT]
}

// [Us][Move], Or rather [Us][To SQ][From SQ]
#[allow(non_camel_case_types)]
type BF_idx = (Player, BitMove);

impl Index<BF_idx> for ButterflyHistory {
    type Output = i16;

    fn index(&self, idx: BF_idx) -> &Self::Output {
        unsafe {
            let from_to = idx.1.from_to() as usize;
            self.a.get_unchecked(idx.0 as usize)   // [player]
                .get_unchecked(from_to)            // [From SQ][to SQ]
        }
    }
}

impl IndexMut<BF_idx> for ButterflyHistory {
    fn index_mut(&mut self, idx: BF_idx) -> &mut Self::Output {
        unsafe {
            let from_to = idx.1.from_to() as usize;
            self.a.get_unchecked_mut(idx.0 as usize)   // [player]
                .get_unchecked_mut(from_to)            // [From SQ][to SQ]
        }
    }
}


impl StatBoard<i16, BF_idx> for ButterflyHistory {
    const FILL: i16 = 0;
}

impl NumStatBoard<BF_idx> for ButterflyHistory {
    const D: i16 = 324;
}
