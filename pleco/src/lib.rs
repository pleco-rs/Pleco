//! A blazingly fast chess library.
//!
//! This package is separated into two parts. Firstly, the board representation & associated functions
//! (the current crate, `pleco`), and secondly, the AI implementations using these chess foundations,
//! [pleco_engine](https://crates.io/crates/pleco_engine).
//!
//! The general formatting and structure of the library take heavy influence from the basic building
//! blocks of the [Stockfish](https://stockfishchess.org/) chess engine.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/pleco) and can be
//! used by adding `pleco` to the dependencies in your project's `Cargo.toml`.
//!
//! # Platforms
//!
//! `pleco` is currently tested and created for use with the `x86_64` instruction set in mind.
//! Currently, there are no guarantees of correct behavior if compiled for a different
//! instruction set.
//!
//! # Safety
//!
//! While generally a safe library, pleco was built with a focus of speed in mind. Usage of methods
//! must be followed carefully, as there are many possible ways to `panic` unexpectedly. Methods
//! with the ability to panic will be documented as such.
//!
//! # Examples
//!
//! You can create a [`Board`] with the starting position like so:
//!
//! ```ignore
//! use pleco::Board;
//! let board = Board::start_pos();
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
//! let mut board = Board::start_pos();
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
//! let board = Board::from_fen(start_position).unwrap();
//! ```
//!
//! [`MoveList`]: core/move_list/struct.MoveList.html
//! [`Board`]: board/struct.Board.html

#![cfg_attr(test, allow(dead_code))]
#![allow(clippy::cast_lossless)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::uninit_assumed_init)]
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate mucow;
extern crate num_cpus;
extern crate rand;
extern crate rayon;

pub mod board;
pub mod bots;
pub mod core;
pub mod helper;
pub mod tools;

pub use board::Board;
pub use core::bitboard::BitBoard;
pub use core::move_list::{MoveList, ScoringMoveList};
pub use core::piece_move::{BitMove, ScoringMove};
pub use core::sq::SQ;
pub use core::{File, Piece, PieceType, Player, Rank};
pub use helper::Helper;

pub mod bot_prelude {
    //! Easy importing of all available bots.
    pub use bots::AlphaBetaSearcher;
    pub use bots::IterativeSearcher;
    pub use bots::JamboreeSearcher;
    pub use bots::MiniMaxSearcher;
    pub use bots::ParallelMiniMaxSearcher;
    pub use bots::RandomBot;

    pub use tools::Searcher;
}
