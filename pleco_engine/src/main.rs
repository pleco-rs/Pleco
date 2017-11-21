extern crate pleco;
extern crate pleco_engine;

use pleco::bot_prelude::{JamboreeSearcher,Searcher};
use pleco::{Board,BitMove};
use pleco::engine::UCILimit;

use pleco_engine::pleco_searcher::PlecoSearcher;

use std::thread;


fn main() {
    let mut s = PlecoSearcher::init(true);
    let mut board = Board::default();

    let mut i = 0;

    while i < 125 && !board.checkmate() && !board.stalemate() {
        board.pretty_print();
        if i % 2 == 1 {
            let mov = JamboreeSearcher::best_move_depth(board.shallow_clone(),4);
            println!("Jamboree searcher: {}",mov);
            board.apply_move(mov);
        } else {
            s.search(&board, &UCILimit::Infinite);
            thread::sleep_ms(10000);
            let mov = s.stop_search();
            println!("Pleco searcher: {}",mov);
            board.apply_move(mov);
        }
        i += 1;
    }

}