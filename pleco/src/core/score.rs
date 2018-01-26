use std::ops::*;

#[derive(Ord, PartialOrd, Eq, PartialEq,Copy, Clone)]
pub struct Value(i16);

impl Value {
    pub const ZERO: Value = Value(0);
    pub const DRAW: Value = Value(0);
    pub const LIKELY_WIN: Value = Value(-10000);
    pub const MATE: Value = Value(32000);
    pub const INFINITE: Value = Value(32001);
    pub const NEG_INFINITE: Value = Value(-32001);
    pub const NONE: Value = Value(32002);

    pub const PAWN: Value = Value(100);
    pub const KNIGHT: Value = Value(350);
    pub const BISHOP: Value = Value(351);
    pub const ROOK: Value = Value(500);
    pub const QUEEN: Value = Value(900);

    pub const PAWN_MG: Value = Value(171);
    pub const KNIGHT_MG: Value = Value(764);
    pub const BISHOP_MG: Value = Value(826);
    pub const ROOK_MG: Value = Value(1282);
    pub const QUEEN_MG: Value = Value(2526);

    pub const PAWN_EG: Value = Value(240);
    pub const KNIGHT_EG: Value = Value(848);
    pub const BISHOP_EG: Value = Value(891);
    pub const ROOK_EG: Value = Value(1373);
    pub const QUEEN_EG: Value = Value(2646);
}

impl_bit_ops!(Value, i16);

#[derive(Copy, Clone)]
pub struct Score {
    pub mg: i16,
    pub eg: i16
}