//! Miscellaneous tools for used for Searching. Most notably this module
//! contains the `TranspositionTable`, a fast lookup table able to be accessed by
//! multiple threads. Other useful objects are the `UciLimit` enum and `Searcher` trait
//! for building bots.

pub mod prng;
pub mod eval;
pub mod tt;
pub mod pleco_arc;

use core::piece_move::BitMove;
use board::Board;

/// Defines an object that can play chess.
pub trait Searcher {
    fn name() -> &'static str where Self: Sized;

    fn best_move(board: Board, depth: u16) -> BitMove
        where
            Self: Sized;
}
