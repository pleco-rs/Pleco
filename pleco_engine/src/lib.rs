//! A Rust re-write of the Stockfish chess engine.
//!
//! This crate is not intended to be used by other crates as a dependency, as it's a mostly useful as a direct
//! executable.
//!
//! If you are interested in using the direct chess library functions (The Boards, move generation, etc), please
//! checkout the core library, `pleco`, available on [on crates.io](https://crates.io/crates/pleco).
//!
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(test, allow(dead_code))]

#![allow(dead_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::mut_from_ref)]
#![allow(clippy::cognitive_complexity)]

#![feature(ptr_internals)]
#![feature(integer_atomics)]
#![feature(test)]
#![feature(allocator_api)]
#![feature(trusted_len)]
#![feature(const_mut_refs)]
#![feature(alloc_layout_extra)]
#![feature(thread_spawn_unchecked)]

//#![crate_type = "staticlib"]

extern crate num_cpus;
extern crate rand;
extern crate pleco;
extern crate chrono;
extern crate prefetch;

pub mod threadpool;
pub mod sync;
pub mod time;
pub mod consts;
pub mod uci;
pub mod root_moves;
pub mod movepick;
pub mod tables;
pub mod engine;
pub mod search;

pub use consts::*;