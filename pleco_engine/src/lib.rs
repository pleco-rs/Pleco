//! A Rust re-write of the Stockfish chess engine.
//!
//! This crate is not intended to be used by other crates as a dependency, as it's a mostly useful as a direct
//! executable.
//!
//! If you are interested in using the direct chess library functions (The Boards, move generation, etc), please
//! checkout the core library, `pleco`, available on [on crates.io](https://crates.io/crates/pleco).
//!
#![cfg_attr(test, allow(dead_code))]
#![allow(dead_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::mut_from_ref)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::uninit_assumed_init)]

//#![crate_type = "staticlib"]

extern crate chrono;
extern crate num_cpus;
extern crate pleco;
extern crate rand;

pub mod consts;
pub mod engine;
pub mod movepick;
pub mod root_moves;
pub mod search;
pub mod sync;
pub mod tables;
pub mod threadpool;
pub mod time;
pub mod uci;

pub use consts::*;
