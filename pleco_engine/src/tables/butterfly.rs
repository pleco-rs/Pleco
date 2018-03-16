
use pleco::core::masks::*;

use super::{StatBoard, NumStatBoard};

/// ButterflyBoards are 2 tables (one for each color) indexed by the move's from
/// and to squares, see chessprogramming.wikispaces.com/Butterfly+Boards
pub struct ButterflyHistory {
    a: [[i16; (SQ_CNT * SQ_CNT)]; PLAYER_CNT]
}

impl StatBoard<i16> for ButterflyHistory {
    const FILL: i16 = 0;
}

impl NumStatBoard for ButterflyHistory {
    const D: i16 = 324;
}