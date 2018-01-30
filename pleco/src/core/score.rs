//! Primitives for determining the value / score of a specific location.
//!
//! A `Value` stores a single `i16` to represent a score. `Score` stores two `i16`s inside of it,
//! the first to determine the mid-game score, and the second to determine the end-game score.

use std::ops::*;

/// Wrapper type for `i16` to determine the `Value` of an evaluation.
#[derive(Ord, PartialOrd, Eq, PartialEq,Copy, Clone)]
pub struct Value(pub i16);

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

/// Struct to define the value of a mid-game / end-game evaluation.
#[derive(Copy, Clone)]
pub struct Score(pub i16, pub i16);

impl Score {
    pub const ZERO: Score = Score(0,0);

    /// Creates a new `Score`.
    pub fn make(mg: i16, eg: i16) -> Self {
        Score(mg, eg)
    }

    /// Creates a new `Score`.
    pub fn new(mg: Value, eg: Value) -> Self {
        Score(mg.0, eg.0)
    }

    /// Returns the mid-game score.
    pub fn mg(self) -> Value {
        Value(self.0)
    }

    /// Returns the end-game score.
    pub fn eg(self) -> Value {
        Value(self.1)
    }
}

impl Add for Score {
    type Output = Score;

    fn add(self, other: Score) -> Score {
        Score(other.0 + self.0, other.1 + self.1)
    }
}


impl AddAssign for Score {
    fn add_assign(&mut self, other: Score) {
        *self = Score(other.0 + self.0, other.1 + self.1);
    }
}

impl Sub for Score {
    type Output = Score;

    fn sub(self, other: Score) -> Score {
        Score(other.0 - self.0, other.1 - self.1)
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, other: Score) {
        *self = Score(other.0 - self.0, other.1 - self.1);
    }
}