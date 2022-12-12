use std::fmt;
use std::mem;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Pick {
    // Main Searching Algo
    MainSearch = 0,
    CapturesInit = 1,
    GoodCaptures = 2,
    KillerOne = 3,
    KillerTwo = 4,
    CounterMove = 5,
    QuietInit = 6,
    QuietMoves = 7,
    BadCaptures = 8,
    // Evasions, Main and Qsearch
    EvasionSearch = 9,
    EvasionsInit = 10,
    AllEvasions = 11,
    // ProbCut Searching
    ProbCutSearch = 12,
    ProbCutCapturesInit = 13,
    ProbCutCaptures = 14,
    // QSearch - Non Re-Captures
    QSearch = 15,
    QSearchInit = 16,
    QCaptures = 17,
    QChecks = 18,
    // Q Search ReCaptures
    QSearchRecaptures = 19,
    QRecaptures = 20,
}

impl Pick {
    pub fn incr(&mut self) {
        unsafe {
            *self = mem::transmute(*self as u8 + 1);
        }
    }
}

impl fmt::Display for Pick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&match self {
            Pick::MainSearch => "MainSearch",
            Pick::CapturesInit => "CapturesInit",
            Pick::GoodCaptures => "GoodCaptures",
            Pick::KillerOne => "KillerOne",
            Pick::KillerTwo => "KillerTwo",
            Pick::CounterMove => "CounterMove",
            Pick::QuietInit => "QuietInit",
            Pick::QuietMoves => "QuietMoves",
            Pick::BadCaptures => "BadCaptures",
            Pick::EvasionSearch => "EvasionSearch",
            Pick::EvasionsInit => "EvasionsInit",
            Pick::AllEvasions => "AllEvasions",
            Pick::ProbCutSearch => "ProbCutSearch",
            Pick::ProbCutCapturesInit => "ProbCutCapturesInit",
            Pick::ProbCutCaptures => "ProbCutCaptures",
            Pick::QSearch => "QSearch",
            Pick::QSearchInit => "QSearchInit",
            Pick::QCaptures => "QCaptures",
            Pick::QChecks => "QChecks",
            Pick::QSearchRecaptures => "QSearchRecaptures",
            Pick::QRecaptures => "QRecaptures",
        })
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
