//! A rust-based UCI compatible chess AI.
//!
//! This crate is not intended to be used by other crates as a dependency, as it's a mostly useful as a direct
//! executable.
//!
//! If you are interested in using the direct chess library functions (The Boards, move generation, etc), please
//! checkout the core library, `pleco`, available on [on crates.io](https://crates.io/crates/pleco).
//!
//! # Usage as a Dependency
//!
//! This crate is [on crates.io](https://crates.io/crates/pleco_engine) and can be
//! used by adding `pleco_engine` to the dependencies in your project's `Cargo.toml`.
//!
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(inline_always))]
#![cfg_attr(feature="clippy", allow(unreadable_literal))]
#![cfg_attr(feature="clippy", allow(large_digit_groups))]
#![cfg_attr(feature="clippy", allow(cast_lossless))]
#![cfg_attr(feature="clippy", allow(doc_markdown))]
#![cfg_attr(feature="clippy", allow(inconsistent_digit_grouping))]

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(test, allow(dead_code))]

#![allow(dead_code)]

#![feature(ptr_internals)]
#![feature(integer_atomics)]
#![feature(test)]
#![feature(allocator_api)]
#![feature(trusted_len)]
#![feature(fused)]
#![feature(const_indexing)]
#![feature(const_fn)]

#![crate_type = "staticlib"]

#[macro_use]
extern crate lazy_static;

extern crate test;
extern crate num_cpus;
extern crate rand;
extern crate pleco;
extern crate chrono;

pub mod searcher;
pub mod sync;
pub mod time;
pub mod consts;
pub mod uci;
pub mod root_moves;
pub mod tables;

pub use consts::*;