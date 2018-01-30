extern crate pleco_engine;


use pleco_engine::pleco_searcher::PlecoSearcher;



fn main() {

    let mut s = PlecoSearcher::init(true);
    s.uci();
}
