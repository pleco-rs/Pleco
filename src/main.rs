extern crate rusty_chess;


fn main() {
    let b = rusty_chess::board::Board::default();
    b.pretty_print();
}
