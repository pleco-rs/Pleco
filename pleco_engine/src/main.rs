extern crate pleco_engine;


use pleco_engine::searcher::PlecoSearcher;



fn main() {

    let mut s = PlecoSearcher::init(true);
    s.uci();
}
