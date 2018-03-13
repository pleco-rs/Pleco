use std::time::Duration;
use criterion::{Criterion,black_box,Bencher};

use pleco::{Board};

use pleco_engine::engine::PlecoSearcher;
use pleco_engine::time::uci_timer::PreLimits;
use pleco_engine::consts::*;
use pleco_engine::threadpool::*;

use super::*;


fn search_3moves_engine<D: DepthLimit>(b: &mut Bencher) {
    let mut pre_limit = PreLimits::blank();
    pre_limit.depth = Some(D::depth());
    let _searcher = PlecoSearcher::init(false);
    let limit = pre_limit.create();
    b.iter_with_setup(|| {
        threadpool().clear_all();
        unsafe {TT_TABLE.clear() };
        Board::start_pos()
    }, |mut board| {
        let mov = black_box(threadpool().search(&board, &limit));
        board.apply_move(mov);
        let mov = black_box(threadpool().search(&board, &limit));
        board.apply_move(mov);
        black_box(threadpool().search(&board, &limit));
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

