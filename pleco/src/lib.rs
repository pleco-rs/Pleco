//! A blazingly fast Chess Library.
//!
//! This package is separated into two parts. Firstly, the board representation & associated functions (the current crate, `pleco`), and secondly,
//! the AI implementations [pleco_engine](https://crates.io/crates/pleco_engine).
//!
//! This crate requires NIGHTLY Rust to use.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/pleco) and can be
//! used by adding `pleco` to the dependencies in your project's `Cargo.toml`.
//!
//! `pleco` requires nightly rust currently, so make sure your toolchain is a nightly version.
//!
//!
//! # Safety
//!
//! While generally a safe library, pleco was built with a focus of speed in mind. Usage of methods must be followed
//! carefully, as there are many possible ways to `panic` unexpectedly. Methods with the ability to panic will be
//! documented as such.
//!
//!
//! # Examples
//!
//! You can create a [`Board`] with the starting position like so:
//!
//! ```ignore
//! use pleco::Board;
//! let board = Board::default();
//! ```
//!
//! Generating a list of moves (Contained inside a [`MoveList`]) can be done with:
//!
//! ```ignore
//! let list = board.generate_moves();
//! ```
//!
//! Applying and undoing moves is simple:
//!
//! ```ignore
//! let mut board = Board::default();
//! let list = board.generate_moves();
//!
//! for mov in list.iter() {
//!     board.apply_move(*mov);
//!     println!("{}",board.get_fen());
//!     board.undo_move();
//! }
//! ```
//!
//! Using fen strings is also supported:
//!
//! ```ignore
//! let start_position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//! let board = Board::new_from_fen(start_position);
//! ```
//!
//! [`MoveList`]: core/move_list/struct.MoveList.html
//! [`Board`]: board/struct.Board.html
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

//#![crate_type = "rlib"]

#![feature(fused)]
#![feature(trusted_len)]
#![feature(test)]
#![feature(integer_atomics)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(const_indexing)]
#![feature(pointer_methods)]
#![feature(cfg_target_feature, target_feature)]
#![feature(stdsimd)]



#![allow(dead_code)]


// [`Vec<T>`]: ../../std/vec/struct.Vec.html
// [`new`]: ../../std/vec/struct.Vec.html#method.new
// [`push`]: ../../std/vec/struct.Vec.html#method.push
// [`Index`]: ../../std/ops/trait.Index.html
// [`IndexMut`]: ../../`std/ops/trait.IndexMut.html
// [`vec!`]: ../../std/macro.vec.html
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate rand;
extern crate rayon;
extern crate prefetch;
extern crate mucow;
extern crate test;
extern crate unreachable;


pub mod core;
pub mod board;
pub mod bots;
pub mod bot_prelude;
pub mod helper;
pub mod tools;

#[doc(no_inline)]
pub use board::Board;
#[doc(no_inline)]
pub use core::piece_move::{BitMove,ScoringMove};
#[doc(no_inline)]
pub use core::move_list::{MoveList,ScoringMoveList};
#[doc(no_inline)]
pub use core::sq::SQ;
#[doc(no_inline)]
pub use core::bitboard::BitBoard;
#[doc(no_inline)]
pub use helper::Helper;
#[doc(no_inline)]
pub use core::{Player, PieceType, Rank, File};


