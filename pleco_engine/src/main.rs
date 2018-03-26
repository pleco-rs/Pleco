extern crate pleco_engine;
use pleco_engine::engine::PlecoSearcher;

fn main() {
    let mut s = PlecoSearcher::init(true);
    s.uci();
}
