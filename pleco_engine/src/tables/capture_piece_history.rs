use std::ops::{Index,IndexMut};
use pleco::core::masks::*;
use pleco::{PieceType, SQ,Player};

use super::{StatBoard,NumStatCube};

/// CapturePieceToBoards are addressed by a move's
/// [player][moved piecetype][to][captured piecetype] information.
pub struct CapturePieceToHistory {
    a: [[[[i16; PIECE_TYPE_CNT]; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}

// [player][moved piecetype][to][captured piecetype]
#[allow(non_camel_case_types)]
type CP_idx = (Player, PieceType, SQ, PieceType);

impl Index<CP_idx> for CapturePieceToHistory {
    type Output = i16;

    fn index(&self, idx: CP_idx) -> &Self::Output {
        unsafe {
            self.a.get_unchecked(idx.0 as usize)    // [player]
                .get_unchecked(idx.1 as usize)      // [Moved piece]
                .get_unchecked((idx.2).0 as usize)  // [to square]
                .get_unchecked(idx.3 as usize)      // [Captured piece]
        }
    }
}

impl IndexMut<CP_idx> for CapturePieceToHistory {
    fn index_mut(&mut self, idx: CP_idx) -> &mut Self::Output {
        unsafe {
            self.a.get_unchecked_mut(idx.0 as usize)    // [player]
                .get_unchecked_mut(idx.1 as usize)      // [Moved piece]
                .get_unchecked_mut((idx.2).0 as usize)  // [to square]
                .get_unchecked_mut(idx.3 as usize)      // [Captured piece]
        }
    }
}

impl StatBoard<i16, CP_idx> for CapturePieceToHistory {
    const FILL: i16 = 0;
}

impl NumStatCube<CP_idx> for CapturePieceToHistory {
    const D: i16 = 324;
    const W: i16 = 2;
}
