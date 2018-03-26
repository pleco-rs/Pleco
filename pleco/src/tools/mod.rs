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
    /// Returns the name of the searcher.
    fn name() -> &'static str where Self: Sized;

    /// Returns the BestMove of a position from a search of depth.
    fn best_move(board: Board, depth: u16) -> BitMove
        where
            Self: Sized;
}

/// Allows an object to have it's entries pre-fetchable.
pub trait PreFetchable {
    /// Pre-fetches a particular key. This means bringing it into the cache for faster access.
    fn prefetch(&self, key: u64);

    /// Pre-fetches a particular key, alongside the next key.
    fn prefetch2(&self, key: u64) {
        self.prefetch(key);
        self.prefetch(key + 1);
    }
}
