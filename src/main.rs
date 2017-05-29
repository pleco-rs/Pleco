extern crate rusty_chess;
use rusty_chess::magic_helper;


fn main() {
    magic_helper::gen_rook_masks();
    println!();
    magic_helper::gen_bishop_masks();
}