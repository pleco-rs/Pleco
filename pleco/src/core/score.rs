//! Primitives for determining the value / score of a specific location.
//!
//! A `Value` stores a single `i16` to represent a score. `Score` stores two `i16`s inside of it,
//! the first to determine the mid-game score, and the second to determine the end-game score.

use std::ops::*;

/// Type for `i16` to determine the `Value` of an evaluation.
pub type Value = i16;


pub const ZERO: Value = 0;
pub const DRAW: Value = 0;
pub const LIKELY_WIN: Value = -10000;
pub const MATE: Value = 31000;
pub const INFINITE: Value = 32001;
pub const NEG_INFINITE: Value = -32001;
pub const NONE: Value = 32002;

pub const PAWN: Value = 100;
pub const KNIGHT: Value = 350;
pub const BISHOP: Value = 351;
pub const ROOK: Value = 500;
pub const QUEEN: Value = 900;

pub const PAWN_MG: Value = 171;
pub const KNIGHT_MG: Value = 764;
pub const BISHOP_MG: Value = 826;
pub const ROOK_MG: Value = 1282;
pub const QUEEN_MG: Value = 2526;

pub const PAWN_EG: Value = 240;
pub const KNIGHT_EG: Value = 848;
pub const BISHOP_EG: Value = 891;
pub const ROOK_EG: Value = 1373;
pub const QUEEN_EG: Value = 2646;


/// Struct to define the value of a mid-game / end-game evaluation.
#[derive(Copy, Clone)]
pub struct Score(pub Value, pub Value);

impl Score {
    pub const ZERO: Score = Score(0,0);

    /// Creates a new `Score`.
    pub fn make(mg: Value, eg: Value) -> Self {
        Score(mg, eg)
    }

    /// Creates a new `Score`.
    pub fn new(mg: Value, eg: Value) -> Self {
        Score(mg, eg)
    }

    /// Returns the mid-game score.
    pub fn mg(self) -> Value {
        self.0
    }

    /// Returns the end-game score.
    pub fn eg(self) -> Value {
        self.1
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