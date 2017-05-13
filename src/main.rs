mod board;
mod lib;
mod piece_move;
mod movegen;
mod templates;
mod bit_twiddles;

include!("../test/test.rs");




fn main() {
    board::main();
}
