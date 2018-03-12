use std::time::Duration;
use criterion::{Criterion,black_box,Bencher};

use pleco::{Board};

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;

use super::*;


fn search_singular_engine<D: DepthLimit>(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(D::depth());
    b.iter_with_setup(|| {
        let mut s = PlecoSearcher::init(false);
        s.clear_search();
        (Board::default(), s)
    }, |(board, mut s)| {
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

fn bench_engine_evaluations(c: &mut Criterion) {
    c.bench_function("Search Singular Depth 3", search_singular_engine::<Depth3>);
    c.bench_function("Search Singular Depth 4", search_singular_engine::<Depth4>);
    c.bench_function("Search Singular Depth 5", search_singular_engine::<Depth5>);
    c.bench_function("Search Singular Depth 6", search_singular_engine::<Depth6>);
    c.bench_function("Search Singular Depth 7", search_singular_engine::<Depth7>);
}

criterion_group!(name = search_singular;
     config = Criterion::default()
        .sample_size(18)
        .warm_up_time(Duration::from_millis(100));
    targets = bench_engine_evaluations
);

