extern crate pleco;

use pleco::bot_prelude::{JamboreeSearcher,Searcher};
use pleco::pleco_searcher::_PlecoSearcher;
use pleco::{Board,BitMove};
use pleco::engine::UCILimit;
use std::env;

use std::thread;

//fn main() {
//    let args: Vec<String> = env::args().collect();
//    console_loop(args);
//}


fn main() {
    let mut s = _PlecoSearcher::init(false);
    let mut board = Board::default();

    let mut i = 0;

    while i < 100 && !board.checkmate() && !board.stalemate() {
        board.pretty_print();
        if i % 2 == 1 {
            let mov = JamboreeSearcher::best_move_depth(board.shallow_clone(),4);
            println!("Jamboree searcher: {}",mov);
            board.apply_move(mov);
        } else {
            s.search(&board, &UCILimit::Infinite);
            thread::sleep_ms(6000);
            let mov = s.stop_search();
            println!("Pleco searcher: {}",mov);
            board.apply_move(mov);
        }
        i += 1;
    }

}