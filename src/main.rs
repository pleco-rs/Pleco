mod board;
mod lib;
mod piece_move;
mod movegen;
mod templates;

include!("../test/test.rs");




fn main() {
    board::main();
}
