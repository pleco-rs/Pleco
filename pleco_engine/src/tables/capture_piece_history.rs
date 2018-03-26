use pleco::core::masks::*;

use super::{StatBoard,NumStatCube};

/// CapturePieceToBoards are addressed by a move's[player][piece][to][captured piece] information
pub struct CapturePieceToHistory {
    a: [[[[i16; PIECE_TYPE_CNT]; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}


impl StatBoard<i16> for CapturePieceToHistory {
    const FILL: i16 = 0;
}

impl NumStatCube for CapturePieceToHistory {
    const D: i16 = 324;
    const W: i16 = 2;
}
