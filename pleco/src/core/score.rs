//! Primitives for determining the value / score of a specific location.
//!
//! A `Value` stores a single `i32` to represent a score. `Score` stores two `i32`s inside of it,
//! the first to determine the mid-game score, and the second to determine the end-game score.

use std::fmt;
use std::ops::*;

// TODO: Why is Value an i32 now? Need some notes on why that changed.

/// Type for `i32` to determine the `Value` of an evaluation.
pub type Value = i32;

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

pub const MID_GAME_LIMIT: Value = 15258;
pub const END_GAME_LIMIT: Value = 3915;

pub const MATE_IN_MAX_PLY: Value = MATE - 2 * 128;
pub const MATED_IN_MAX_PLY: Value = -MATE + 2 * 128;

/// Struct to define the value of a mid-game / end-game evaluation.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Score(pub Value, pub Value);

impl Score {
    /// The Score of zero
    pub const ZERO: Score = Score(0, 0);

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

    /// Gives the value of the score in centi-pawns
    pub fn centipawns(self) -> (f64, f64) {
        let mg: f64 = self.mg() as f64 / PAWN_MG as f64;
        let eg: f64 = self.eg() as f64 / PAWN_EG as f64;
        (mg, eg)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (mg, eg) = self.centipawns();
        write!(f, "{:5.2} {:5.2}", mg, eg)
    }
}

impl Add for Score {
    type Output = Score;

    fn add(self, other: Score) -> Score {
        Score(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Score) {
        *self = Score(self.0 + other.0, self.1 + other.1);
    }
}

impl Sub for Score {
    type Output = Score;

    fn sub(self, other: Score) -> Score {
        Score(self.0 - other.0, self.1 - other.1)
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, other: Score) {
        *self = Score(self.0 - other.0, self.1 - other.1);
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Score {
        Score(-self.0, -self.1)
    }
}

impl Mul<u8> for Score {
    type Output = Score;

    fn mul(self, rhs: u8) -> Score {
        Score(self.0 * rhs as i32, self.1 * rhs as i32)
    }
}

impl Mul<u16> for Score {
    type Output = Score;

    fn mul(self, rhs: u16) -> Score {
        Score(self.0 * rhs as i32, self.1 * rhs as i32)
    }
}

impl Mul<i16> for Score {
    type Output = Score;

    fn mul(self, rhs: i16) -> Score {
        Score(self.0 * rhs as i32, self.1 * rhs as i32)
    }
}

impl Mul<i32> for Score {
    type Output = Score;

    fn mul(self, rhs: i32) -> Score {
        Score(self.0 * rhs, self.1 * rhs)
    }
}
