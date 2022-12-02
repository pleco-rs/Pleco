use pleco::core::masks::*;
use pleco::{Piece, PieceType, SQ};
use std::ops::{Index, IndexMut};

use super::{NumStatCube, StatBoard};

/// CapturePieceToBoards are addressed by a move's
/// [player][moved piecetype][to][captured piecetype] information.
pub struct CapturePieceToHistory {
    a: [[[i16; PIECE_TYPE_CNT]; SQ_CNT]; PIECE_CNT],
}

// [player][moved piecetype][to][captured piecetype]
#[allow(non_camel_case_types)]
type CP_idx = (Piece, SQ, PieceType);

impl Index<CP_idx> for CapturePieceToHistory {
    type Output = i16;

    #[inline(always)]
    fn index(&self, idx: CP_idx) -> &Self::Output {
        unsafe {
            self.a
                .get_unchecked(idx.0 as usize) // [Moved Piece]
                .get_unchecked((idx.1).0 as usize) // [to square]
                .get_unchecked(idx.2 as usize) // [Captured piece type]
        }
    }
}

impl IndexMut<CP_idx> for CapturePieceToHistory {
    #[inline(always)]
    fn index_mut(&mut self, idx: CP_idx) -> &mut Self::Output {
        unsafe {
            self.a
                .get_unchecked_mut(idx.0 as usize) // [Moved Piece]
                .get_unchecked_mut((idx.1).0 as usize) // [to square]
                .get_unchecked_mut(idx.2 as usize) // [Captured piece type]
        }
    }
}

impl StatBoard<i16, CP_idx> for CapturePieceToHistory {
    const FILL: i16 = 0;
}

impl NumStatCube<CP_idx> for CapturePieceToHistory {
    const D: i32 = 324;
    const W: i32 = 2;
}
