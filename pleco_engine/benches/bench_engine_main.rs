#[macro_use]
extern crate criterion;

extern crate pleco;
extern crate pleco_engine;

mod eval_benches;
mod multimove_benches;
mod startpos_benches;

trait DepthLimit {
    fn depth() -> u16;
}

struct Depth5 {}
struct Depth6 {}
struct Depth7 {}
struct Depth8 {}
struct Depth9 {}

impl DepthLimit for Depth5 {
    fn depth() -> u16 {
        5
    }
}
impl DepthLimit for Depth6 {
    fn depth() -> u16 {
        6
    }
}
impl DepthLimit for Depth7 {
    fn depth() -> u16 {
        7
    }
}
impl DepthLimit for Depth8 {
    fn depth() -> u16 {
        8
    }
}
impl DepthLimit for Depth9 {
    fn depth() -> u16 {
        9
    }
}

criterion_main! {
    eval_benches::eval_benches,
    multimove_benches::search_multimove,
    startpos_benches::search_singular
}
