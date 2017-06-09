#![feature(test)]

extern crate rusty_chess;
extern crate test;

mod bit_manipulations;
mod board_build;
mod init_move_generating;
mod fen_building;
mod board_move_apply;
mod magic;

#[cfg(test)]
mod test {
    use super::*;
    use test::Bencher;
}
