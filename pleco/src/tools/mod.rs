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
pub fn prefetch_write<T>(ptr: *const T) {
    __prefetch_write::<T>(ptr);
}


#[cfg(feature = "nightly")]
fn __prefetch_write<T>(ptr: *const T) {
    use std::intrinsics::prefetch_write_data;
    unsafe {
        prefetch_write_data::<T>(ptr, 3);
    }
}

#[cfg(
    all(
        not(feature = "nightly"),
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse"
    )
)]
fn __prefetch_write<T>(ptr: *const T) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_prefetch;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_prefetch;
    unsafe {
        _mm_prefetch(ptr as *const i8, 3);
    }
}

#[cfg(
    all(
        not(feature = "nightly"),
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                not(target_feature = "sse")
            ),
            not(any(target_arch = "x86", target_arch = "x86_64"))
        )
    )
)]
fn __prefetch_write<T>(ptr: *const T) {
    // Do nothing
}