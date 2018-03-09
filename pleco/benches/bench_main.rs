#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;

extern crate pleco;



mod bit_benches;
mod board_benches;
mod bot_benches;
mod eval_benches;
mod magic_benches;
mod move_gen_benches;
mod perft_benches;
mod piece_loc_benches;
mod tt_benches;

criterion_main!{
    bit_benches::bit_benches,
    board_benches::board_benches,
    bot_benches::bot_benches
}