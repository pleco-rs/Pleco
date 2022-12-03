//! Miscellaneous tools for used for Searching. Most notably this module
//! contains the `TranspositionTable`, a fast lookup table able to be accessed by
//! multiple threads. Other useful objects are the `UciLimit` enum and `Searcher` trait
//! for building bots.

pub mod eval;
pub mod pleco_arc;
pub mod prng;
pub mod tt;

use board::Board;
use core::piece_move::BitMove;

/// Defines an object that can play chess.
pub trait Searcher {
    /// Returns the name of the searcher.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Returns the BestMove of a position from a search of depth.
    fn best_move(board: Board, depth: u16) -> BitMove
    where
        Self: Sized;
}

// https://doc.rust-lang.org/core/arch/x86_64/fn._mm_prefetch.html
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

/// Prefetch's `ptr` to all levels of the cache.
///
/// For some platforms this may compile down to nothing, and be optimized away.
/// To prevent compiling down into nothing, compilation must be done for a
/// `x86` or `x86_64` platform with SSE instructions available. An easy way to
/// do this is to add the environmental variable `RUSTFLAGS=-C target-cpu=native`.
#[inline(always)]
pub fn prefetch_write<T>(ptr: *const T) {
    __prefetch_write::<T>(ptr);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse"
))]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_prefetch;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_prefetch;
    unsafe {
        _mm_prefetch(ptr as *const i8, 3);
    }
}

#[cfg(all(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_feature = "sse")
    ),
    not(any(target_arch = "x86", target_arch = "x86_64"))
)))]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    // Do nothing
}

/// Hints to the compiler for optimizations.
///
/// These functions normally compile down to no-operations without the `nightly` flag.
pub mod hint {

    /// Hints to the compiler that branch condition is likely to be false.
    /// Returns the value passed to it.
    ///
    /// Any use other than with `if` statements will probably not have an effect.
    #[inline(always)]
    pub fn unlikely(cond: bool) -> bool {
        cond
    }

    /// Hints to the compiler that branch condition is likely to be true.
    /// Returns the value passed to it.
    ///
    /// Any use other than with `if` statements will probably not have an effect.
    #[inline(always)]
    pub fn likely(cond: bool) -> bool {
        cond
    }
}
