#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

pub mod board;
pub mod bit_twiddles;
pub mod movegen;
pub mod piece_move;
pub mod templates;



//include!("tests/test.rs");