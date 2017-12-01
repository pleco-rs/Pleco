//! Miscellaneous tools for used for Searching. Most notably this module
//! contains the `TranspositionTable`, a fast lookup table able to be accessed by
//! multiple threads. Other useful objects are the `UciLimit` enum and `Searcher` trait
//! for building bots.

pub mod prng;
pub mod tt;
pub mod timer;

use core::piece_move::BitMove;
use tools::timer::Timer;
use board::Board;

/// Defines an object that can play chess.
pub trait Searcher {
    fn name() -> &'static str where Self: Sized;

    fn best_move(board: Board, limit: UCILimit) -> BitMove
        where
            Self: Sized;

    fn best_move_depth(board: Board, max_depth: u16) -> BitMove
        where
            Self: Sized {
        Self::best_move(board, UCILimit::Depth(max_depth))
    }
}


/// Defines a Limit for a Searcher. e.g., when a searcher should stop
/// searching.
#[derive(Clone)]
pub enum UCILimit {
    Infinite,
    Depth(u16),
    Nodes(u64),
    Time(Timer),
}

impl UCILimit {
    /// Returns if time management should be used.
    pub fn use_time(&self) -> bool {
        if let UCILimit::Time(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns if the limit is depth.
    pub fn is_depth(&self) -> bool {
        if let UCILimit::Depth(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns the depth limit if there is one, otherwise returns 10000.
    pub fn depth_limit(&self) -> u16 {
        if let UCILimit::Depth(depth) = *self {
            depth
        } else {
            10_000
        }
    }

    /// Returns the Timer for the UCILimit, if there is one to be sent.
    pub fn timer(&self) -> Option<Timer> {
        if let UCILimit::Time(timer) = *self {
            Some(timer.clone())
        } else {
            None
        }
    }
}
