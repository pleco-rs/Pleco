use std::mem;

pub trait PickInner {
    fn incr(&mut self);
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PickMain {
    MainSearch = 0,
    CapturesInit = 1,
    GoodCaptures = 2,
    KillerOne = 3,
    KillerTwo = 4,
    CounterMove = 5,
    QuietInit = 6,
    QuietMoves = 7,
    BadCaptures = 8,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PickEvasion {
    EvasionSearch = 0,
    EvasionsInit = 1,
    AllEvasions = 2,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PickProbCut {
    ProbCutSearch = 0,
    ProbCutCapturesInit = 1,
    ProbCutCaptures = 2,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PickQSearch {
    QSearch = 0,
    QSearchInit = 1,
    QCaptures = 2,
    QChecks = 3,
    QSearchRecaptures = 4,
    QRecaptures = 5,
}

impl PickInner for PickMain {
    fn incr(&mut self) {
        unsafe {*self = mem::transmute(*self as u8 + 1); }
    }
}

impl PickInner for PickEvasion {
    fn incr(&mut self) {
        unsafe {*self = mem::transmute(*self as u8 + 1); }
    }
}

impl PickInner for PickProbCut {
    fn incr(&mut self) {
        unsafe {*self = mem::transmute(*self as u8 + 1); }
    }
}

impl PickInner for PickQSearch {
    fn incr(&mut self) {
        unsafe {*self = mem::transmute(*self as u8 + 1); }
    }
}

// types

// Root
// MainSearch
// Evasions
// ProbCut
// Qsearch


// Strategy

// RootMoves -------
// Get the next rootmoves.

// MainSearch ------
// TT Move
//      Increment.
//      Return TT move
// Captures_init
//      Generate Captures
//      Sort<Captures>
//      Increment
//      Go to next_move();
// Good_Captures
//      Loop through each capture, once done increment stage
// Killer0
//      Do KillerMove1, increment
// Killer1
//      Do KillerMove2, increment
// CounterMove
//      Do CounterMove, increment
// Quiet_Init
//      Generate Quiets
//      Sort<Quiets>
//      Partial Insertion sort?
//      Increment
//      Go to next_move();
// Quiet
//
// Bad Captures
//

// Evasions -------
// TT Move
//      Return TT move, Increment.
// Evasions_init
// All_evasions

// ProbCut
// TT Move
//      Return TT move, Increment.
// Probcut_Captures_Init
// Probvut Captures

// Qsearch -----------
// TT Move
//      Return TT move, Increment.
// QCaptures_Init
// QCaptures
// QChecks
// QSearch_Recaptures
// QRecaptures