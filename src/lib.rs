#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(test, allow(dead_code))]

#![feature(test)]
#![allow(dead_code)]
#![feature(integer_atomics)]
#![feature(future_atomic_orderings)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate chashmap;
extern crate chrono;
extern crate test;
extern crate rayon;
extern crate futures;
extern crate parking_lot;
extern crate owning_ref;


pub mod board;
pub mod bit_twiddles;
pub mod movegen;
pub mod piece_move;
pub mod templates;
pub mod magic_helper;
pub mod timer;
pub mod engine;
pub mod transposition_table;


pub mod bots;

pub mod eval;




//include!("tests/test.rs");

