use std::time::Duration;
use criterion::{Criterion,black_box,Bencher};

use pleco::{Board};

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;

use super::*;


fn search_3moves_engine<D: DepthLimit>(b: &mut Bencher) {
    let mut limit = PreLimits::blank();
    limit.depth = Some(D::depth());
    b.iter_with_setup(|| {
        let mut s = PlecoSearcher::init(false);
        s.clear_search();
        (Board::default(), s)
    }, |(mut board, mut s)| {
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        let mov = black_box(s.await_move());
        board.apply_move(mov);
        black_box(s.search(&board, &limit));
        black_box(s.await_move());
    })
}

fn bench_engine_evaluations(c: &mut Criterion) {
    c.bench_function("Search MuliMove Depth 4", search_3moves_engine::<Depth4>);
    c.bench_function("Search MuliMove Depth 5", search_3moves_engine::<Depth5>);
    c.bench_function("Search MuliMove Depth 6", search_3moves_engine::<Depth6>);
    c.bench_function("Search MuliMove Depth 7", search_3moves_engine::<Depth7>);
}

criterion_group!(name = search_multimove;
     config = Criterion::default()
        .sample_size(12)
        .warm_up_time(Duration::from_millis(200));
    targets = bench_engine_evaluations
);

