#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![feature(test)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate chashmap;

extern crate test;
pub mod board;
pub mod bit_twiddles;
pub mod movegen;
pub mod piece_move;
pub mod templates;
pub mod magic_helper;
pub mod transposition_table;



//include!("tests/test.rs");

