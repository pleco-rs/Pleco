extern crate rusty_chess;
use rusty_chess::magic_helper;
use rusty_chess::templates;



fn main() {
    let m = magic_helper::MagicHelper::new();
    templates::print_bitboard(m.rook_moves(0,0));
    templates::print_bitboard(m.queen_moves(0x16060590A2281,63));
    templates::print_bitboard(m.knight_moves(0));
    templates::print_bitboard(m.king_moves(0))
}