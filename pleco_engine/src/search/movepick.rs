
#[allow(unused_imports)]
use pleco::{BitMove,Board};
#[allow(unused_imports)]
use pleco::core::move_list::ScoringMoveList;


// TODO: use Generators once stabilized.

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




pub struct MovePicker {

}
