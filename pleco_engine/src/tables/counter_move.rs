use pleco::core::masks::*;
use pleco::{BitMove};
use super::StatBoard;

/// PieceToBoards are addressed by a move's [player][piece][square] information
pub struct CounterMoveHistory {
    a: [[[BitMove; SQ_CNT]; PIECE_TYPE_CNT]; PLAYER_CNT]
}
impl StatBoard<BitMove> for CounterMoveHistory {
    const FILL: BitMove = BitMove::null();
}
