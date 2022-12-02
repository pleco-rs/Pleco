#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;

extern crate pleco;

mod bit_benches;
mod board_benches;
mod bot_benches;
mod eval_benches;
mod lookup_benches;
mod move_gen_benches;
mod perft_benches;

criterion_main! {
    move_gen_benches::movegen_benches,
    bit_benches::bit_benches,
    board_benches::board_benches,
    bot_benches::bot_benches,
    eval_benches::eval_benches,
    lookup_benches::lookup_benches,
    perft_benches::perft_benches
}
