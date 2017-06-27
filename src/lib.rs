#![cfg_attr(feature="clippy", feature(plugin))]
#![feature(type_ascription)]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(test)]
#![allow(dead_code)]

extern crate test;
pub mod board;
pub mod bit_twiddles;
pub mod movegen;
pub mod piece_move;
pub mod templates;
pub mod fen;
pub mod magic_helper;



//include!("tests/test.rs");

