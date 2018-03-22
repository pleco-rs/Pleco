use std::mem;

use pleco::core::masks::*;
use super::{StatBoard,NumStatBoard};

/// PieceToBoards are addressed by a move's [player][piece][to] information
pub struct PieceToHistory {
    a: [[[i16; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}

impl StatBoard<i16> for PieceToHistory {
    const FILL: i16 = 0;
}

impl NumStatBoard for PieceToHistory {
    const D: i16 = 936;
}


/// ContinuationHistory is the history of a given pair of moves, usually the
/// current one given a previous one. History table is based on PieceToBoards
/// instead of ButterflyBoards.
pub struct ContinuationHistory {
    a: [[[PieceToHistory; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}

impl ContinuationHistory {
    pub fn new() -> Self {
        unsafe {mem::zeroed()}
    }

    pub fn clear(&mut self) {
        *self = unsafe {mem::zeroed()};
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn size() {
        println!("{}", mem::size_of::<ContinuationHistory>());
    }
}